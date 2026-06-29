import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api, exportRingBackup, exportAIReport, postSyncImport } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useChatStore } from '../../stores/chat-store'
import { useInviteStore } from '../../stores/invite-store'
import { LLMConfigForm, ResultMsg } from '../common/LLMConfigForm'
import { useLLMTest, type LLMFormState } from '../common/llm-utils'
import { ConfirmModal } from '../common/ConfirmModal'
import { PromptModal } from '../common/PromptModal'

const smallBtn: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 3,
  padding: '4px 10px',
  fontSize: 10,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const session_mode = useChatStore((s) => s.session_mode)
  const setSessionMode = useChatStore((s) => s.setSessionMode)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)
  const [editing, setEditing] = useState(false)
  const [editForm, setEditForm] = useState<LLMFormState>({ provider: 'openai', model: '', api_key: '', base_url: '' })
  const { testing, result: testResult, setResult: setTestResult, test: runTest } = useLLMTest()
  const [saveError, setSaveError] = useState<string | null>(null)
  const [autoCompact, setAutoCompact] = useState<boolean | null>(null)
  const [autoCompactError, setAutoCompactError] = useState<string | null>(null)
  const [syncing, setSyncing] = useState(false)
  const [syncResult, setSyncResult] = useState<string | null>(null)
  const [confirmDialog, setConfirmDialog] = useState<{ title: string; message: string; action: () => void; variant?: 'danger' | 'default' } | null>(null)
  const [promptDialog, setPromptDialog] = useState<{ title: string; placeholder: string; action: (value: string) => void } | null>(null)
  const [now, setNow] = useState(() => Date.now())

  const tokens = useInviteStore((s) => s.tokens)
  const join_requests = useInviteStore((s) => s.join_requests)
  const fetch_tokens = useInviteStore((s) => s.fetch_tokens)
  const revoke_token = useInviteStore((s) => s.revoke_token)
  const fetch_requests = useInviteStore((s) => s.fetch_requests)
  const approve_request = useInviteStore((s) => s.approve_request)
  const reject_request = useInviteStore((s) => s.reject_request)
  const open_modal = useInviteStore((s) => s.open_modal)

  const active_ring = rings.find((r) => r.id === active_ring_id)
  const is_admin = active_ring?.role === 'creator' || active_ring?.role === 'admin'

  const loadLlm = () => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => {
        setLlmConfig({ ...res, provider: res.provider as LLMProvider })
        if (!editing) {
          setEditForm({ provider: res.provider, model: res.model, api_key: '', base_url: res.base_url || '' })
        }
      })
      .catch(() => {})
  }

  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => { loadLlm() }, [])

  useEffect(() => {
    api.get<{ auto_compact: boolean }>('/config/auto_compact')
      .then((res) => setAutoCompact(res.auto_compact))
      .catch(() => {})
  }, [])

  useEffect(() => {
    const interval = window.setInterval(() => setNow(Date.now()), 60000)
    return () => window.clearInterval(interval)
  }, [])

  useEffect(() => {
    if (!active_ring_id) return
    api.get<{ members: Member[] }>(`/rings/${active_ring_id}/members`)
      .then((res) => setMembers(res.members))
      .catch(() => {})
    if (is_admin) {
      fetch_tokens(active_ring_id)
      fetch_requests(active_ring_id)
    }
  }, [active_ring_id, is_admin, fetch_tokens, fetch_requests])

  const startEdit = () => {
    if (!llmConfig) return
    setEditForm({ provider: llmConfig.provider, model: llmConfig.model, api_key: '', base_url: llmConfig.base_url || '' })
    setTestResult(null)
    setSaveError(null)
    setEditing(true)
  }

  const cancelEdit = () => {
    setEditing(false)
    setTestResult(null)
    setSaveError(null)
  }

  const handleSave = async () => {
    setSaveError(null)
    try {
      const body: Record<string, string> = {
        provider: editForm.provider,
        model: editForm.model,
      }
      if (editForm.api_key) body.api_key = editForm.api_key
      if (editForm.base_url) body.base_url = editForm.base_url
      await api.put<LLMConfig>('/config/llm', body)
      setEditing(false)
      loadLlm()
    } catch (e: unknown) {
      setSaveError(e instanceof Error ? e.message : 'Save failed')
    }
  }

  const time_remaining = (expires_at: string) => {
    const diff = new Date(expires_at).getTime() - now
    if (diff <= 0) return 'expired'
    const hours = Math.floor(diff / 3600000)
    if (hours > 24) return `${Math.floor(hours / 24)}d left`
    return `${hours}h left`
  }

  return (
    <div style={{ fontSize: 12 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
        <span style={{ color: 'var(--text-secondary)', fontWeight: 700 }}>LLM Config</span>
        {!editing && (
          <span style={{ ...smallBtn, color: 'var(--accent-cyan)', borderColor: 'var(--accent-cyan)' }} onClick={startEdit}>EDIT</span>
        )}
      </div>

      {!editing && llmConfig && (
        <div style={{ marginBottom: 16, color: 'var(--text-primary)', lineHeight: 1.8 }}>
          <div>Provider: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.provider}</span></div>
          <div>Model: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.model}</span></div>
          <div>API Key: {llmConfig.api_key_set ? '✓' : '✗'}</div>
        </div>
      )}

      <div style={{ marginBottom: 16, padding: '8px 0', borderTop: '1px solid var(--border)', borderBottom: '1px solid var(--border)' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ color: 'var(--text-secondary)', fontWeight: 700 }}>Auto Compact</span>
          <button
            onClick={() => {
              const prev = autoCompact
              const next = !prev
              setAutoCompact(next)
              setAutoCompactError(null)
              api.put('/config/auto_compact', { auto_compact: next })
                .catch(() => {
                  setAutoCompact(prev)
                  setAutoCompactError('Failed to update. Please try again.')
                })
            }}
            style={{
              background: autoCompact ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: autoCompact ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '4px 10px',
              fontSize: 10,
              cursor: 'pointer',
              fontWeight: 700,
            }}
          >
            {autoCompact === null ? '...' : autoCompact ? 'ON' : 'OFF'}
          </button>
        </div>
        {autoCompactError && (
          <div style={{ fontSize: 10, color: 'var(--accent-amber)', marginTop: 4 }}>
            {autoCompactError}
          </div>
        )}
        <div style={{ fontSize: 10, color: 'var(--text-dim)', marginTop: 4 }}>
          Automatically compact history when token threshold is reached
        </div>
      </div>

      {editing && (
        <div style={{ marginBottom: 16 }}>
          <LLMConfigForm value={editForm} onChange={(p) => setEditForm((prev) => ({ ...prev, ...p }))} idPrefix="cfg" />

          <button
            onClick={() => runTest(editForm)}
            disabled={testing}
            style={{
              ...smallBtn,
              width: '100%',
              marginBottom: 8,
              marginTop: 4,
              border: '1px solid var(--accent-cyan)',
              color: 'var(--accent-cyan)',
              background: 'transparent',
              opacity: testing ? 0.5 : 1,
            }}
          >
            {testing ? 'TESTING...' : 'TEST CONNECTION'}
          </button>

          <ResultMsg result={testResult} />

          {saveError && (
            <div style={{
              fontSize: 10,
              padding: '4px 8px',
              borderRadius: 3,
              marginBottom: 8,
              background: 'rgba(239,68,68,0.1)',
              color: 'var(--accent-red, #ef4444)',
              border: '1px solid var(--accent-red, #ef4444)',
            }}>
              {saveError}
            </div>
          )}

          <div style={{ display: 'flex', gap: 6 }}>
            <span style={{ ...smallBtn, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderColor: 'var(--accent-cyan)' }} onClick={handleSave}>SAVE</span>
            <span style={smallBtn} onClick={cancelEdit}>CANCEL</span>
          </div>
        </div>
      )}

      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        Members
      </p>
      {members.map((m) => (
        <div key={m.token_id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 0', color: 'var(--text-primary)' }}>
          <span>{m.display_name}</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>({m.role})</span>
          {m.online && <span style={{ color: 'var(--accent-green)', fontSize: 10 }}>●</span>}
        </div>
      ))}
      {members.length === 0 && <p style={{ color: 'var(--text-dim)' }}>No members</p>}

      {is_admin && (
        <div
          style={{ marginTop: 8, padding: '5px 8px', border: '1px solid var(--accent-cyan)', borderRadius: 3, textAlign: 'center', color: 'var(--accent-cyan)', cursor: 'pointer', fontSize: 10 }}
          onClick={open_modal}
        >
          + invite member
        </div>
      )}

      {is_admin && tokens.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Active Invites · {tokens.length}
          </p>
          {tokens.map((t) => (
            <div key={t.token} style={{ padding: '6px 8px', border: '1px solid var(--border)', borderRadius: 3, marginBottom: 3, display: 'flex', alignItems: 'center', gap: 6, fontSize: 10 }}>
              <span style={{ color: t.type === 'open' ? 'var(--accent-cyan)' : 'var(--accent-amber)', fontSize: 9 }}>{t.type}</span>
              <span style={{ flex: 1, color: 'var(--text-muted)', fontSize: 9 }}>{t.use_count}/{t.max_uses} uses · {time_remaining(t.expires_at)}</span>
               <span style={{ color: 'var(--text-dim)', fontSize: 9, cursor: 'pointer' }} onClick={() => {
                  setConfirmDialog({
                    title: 'Revoke Invite',
                    message: 'Revoke this invite? Anyone with the link will lose access.',
                    variant: 'danger',
                    action: () => { revoke_token(active_ring_id!, t.token) },
                  })
                }}>revoke</span>
            </div>
          ))}
        </>
      )}

      {is_admin && join_requests.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Pending Requests · {join_requests.length}
          </p>
          {join_requests.map((req) => (
            <div key={req.id} style={{ padding: 8, border: '1px solid var(--accent-amber)', borderRadius: 3, marginBottom: 3 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4, fontSize: 10 }}>
                <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>{req.display_name}</span>
                <span style={{ color: 'var(--accent-amber)', fontSize: 8 }}>audit</span>
              </div>
              {req.message && <div style={{ color: 'var(--text-muted)', fontSize: 9, marginBottom: 6 }}>"{req.message}"</div>}
              <div style={{ display: 'flex', gap: 6 }}>
                <div style={{ flex: 1, padding: 4, background: 'var(--accent-green)', color: 'var(--bg-base)', borderRadius: 2, textAlign: 'center', fontSize: 9, fontWeight: 700, cursor: 'pointer' }} onClick={() => approve_request(active_ring_id!, req.id)}>APPROVE</div>
                <div style={{ flex: 1, padding: 4, border: '1px solid var(--border)', borderRadius: 2, textAlign: 'center', fontSize: 9, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={() => {
                  setPromptDialog({
                    title: 'Reject Request',
                    placeholder: 'Rejection reason (optional):',
                    action: (note) => { reject_request(active_ring_id!, req.id, note || undefined) },
                  })
                }}>REJECT</div>
              </div>
            </div>
          ))}
        </>
      )}

      <div style={{ marginTop: 24, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
        <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
          Export
        </p>
        <button
          onClick={() => active_ring_id && exportRingBackup(active_ring_id)}
          style={{
            ...smallBtn,
            width: '100%',
            marginBottom: 4,
          }}
        >
          Full Ring Backup (JSON)
        </button>
        <button
          onClick={async () => {
            if (!active_ring_id) return
            try {
              await exportAIReport(active_ring_id, [], undefined)
            } catch {
              // silently ignore
            }
          }}
          style={{
            ...smallBtn,
            width: '100%',
            marginBottom: 4,
          }}
        >
          AI Report (Markdown)
        </button>
      </div>

      <div style={{ marginTop: 16, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
        <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
          Chat Mode
        </p>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={() => setSessionMode('storage')}
            style={{
              ...smallBtn,
              flex: 1,
              background: session_mode === 'storage' ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: session_mode === 'storage' ? 'var(--bg-base)' : 'var(--text-primary)',
            }}
          >
            Storage
          </button>
          <button
            onClick={() => setSessionMode('ephemeral')}
            style={{
              ...smallBtn,
              flex: 1,
              background: session_mode === 'ephemeral' ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: session_mode === 'ephemeral' ? 'var(--bg-base)' : 'var(--text-primary)',
            }}
          >
            Ephemeral
          </button>
        </div>
        <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 4 }}>
          {session_mode === 'storage' ? 'Messages saved to local database' : 'Messages not saved (temporary)'}
        </div>
      </div>

      {active_ring && active_ring.role !== 'creator' && active_ring.creator_ip && (
        <div style={{ marginTop: 16, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
          <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Sync
          </p>
          <button
            onClick={async () => {
              if (!active_ring_id || !active_ring.creator_ip) return
              setSyncing(true)
              setSyncResult(null)
              try {
                const res = await postSyncImport(active_ring_id, active_ring.creator_ip)
                setSyncResult(
                  `Synced: ${res.imported.nodes} nodes, ${res.imported.edges} edges, ${res.imported.archive_records} archives, ${res.imported.group_docs} docs`
                )
              } catch (e: unknown) {
                const msg = e instanceof Error ? e.message : typeof e === 'string' ? e : JSON.stringify(e)
                setSyncResult(`Sync failed: ${msg}`)
              } finally {
                setSyncing(false)
              }
            }}
            disabled={syncing}
            style={{
              ...smallBtn,
              width: '100%',
              marginBottom: 4,
              cursor: syncing ? 'not-allowed' : 'pointer',
              opacity: syncing ? 0.6 : 1,
            }}
          >
            {syncing ? 'Syncing...' : 'Sync from Creator'}
          </button>
          {syncResult && (
            <div style={{ fontSize: 10, color: 'var(--text-muted)', marginTop: 4 }}>
              {syncResult}
            </div>
          )}
        </div>
      )}
      <ConfirmModal
        open={confirmDialog !== null}
        title={confirmDialog?.title ?? ''}
        message={confirmDialog?.message ?? ''}
        variant={confirmDialog?.variant}
        on_confirm={() => { confirmDialog?.action(); setConfirmDialog(null) }}
        on_cancel={() => setConfirmDialog(null)}
      />
      <PromptModal
        open={promptDialog !== null}
        title={promptDialog?.title ?? ''}
        placeholder={promptDialog?.placeholder}
        on_submit={(value) => { promptDialog?.action(value); setPromptDialog(null) }}
        on_cancel={() => setPromptDialog(null)}
      />
    </div>
  )
}
