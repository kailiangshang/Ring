import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api, testLLMConfig, exportRingBackup, exportAIReport, postSyncImport } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useChatStore } from '../../stores/chat-store'
import { useInviteStore } from '../../stores/invite-store'

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '6px 10px',
  color: 'var(--text-primary)',
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 8,
  marginTop: 2,
}

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
  const [editProvider, setEditProvider] = useState<string>('openai')
  const [editModel, setEditModel] = useState('')
  const [editApiKey, setEditApiKey] = useState('')
  const [editBaseUrl, setEditBaseUrl] = useState('')
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)
  const [autoCompact, setAutoCompact] = useState<boolean | null>(null)
  const [autoCompactError, setAutoCompactError] = useState<string | null>(null)
  const [syncing, setSyncing] = useState(false)
  const [syncResult, setSyncResult] = useState<string | null>(null)

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
          setEditProvider(res.provider)
          setEditModel(res.model)
          setEditBaseUrl(res.base_url || '')
        }
      })
      .catch(() => {})
  }

  useEffect(() => { loadLlm() }, [])

  useEffect(() => {
    api.get<{ auto_compact: boolean }>('/config/auto_compact')
      .then((res) => setAutoCompact(res.auto_compact))
      .catch(() => {})
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
  }, [active_ring_id, is_admin])

  const startEdit = () => {
    if (!llmConfig) return
    setEditProvider(llmConfig.provider)
    setEditModel(llmConfig.model)
    setEditBaseUrl(llmConfig.base_url || '')
    setEditApiKey('')
    setTestResult(null)
    setSaveError(null)
    setEditing(true)
  }

  const cancelEdit = () => {
    setEditing(false)
    setTestResult(null)
    setSaveError(null)
  }

  const handleTest = async () => {
    setTesting(true)
    setTestResult(null)
    try {
      const result = await testLLMConfig({
        provider: editProvider,
        model: editModel,
        api_key: editApiKey || undefined,
        base_url: editBaseUrl || undefined,
      })
      setTestResult(result)
    } catch (e: unknown) {
      setTestResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setTesting(false)
    }
  }

  const handleSave = async () => {
    setSaveError(null)
    try {
      const body: Record<string, string> = {
        provider: editProvider,
        model: editModel,
      }
      if (editApiKey) body.api_key = editApiKey
      if (editBaseUrl) body.base_url = editBaseUrl
      await api.put<LLMConfig>('/config/llm', body)
      setEditing(false)
      loadLlm()
    } catch (e: unknown) {
      setSaveError(e instanceof Error ? e.message : 'Save failed')
    }
  }

  const time_remaining = (expires_at: string) => {
    const diff = new Date(expires_at).getTime() - Date.now()
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
          <div style={{ display: 'flex', gap: 4, marginBottom: 10 }}>
            {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
              <button
                key={p}
                onClick={() => setEditProvider(p)}
                style={{
                  background: editProvider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: editProvider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '4px 10px',
                  fontSize: 10,
                  cursor: 'pointer',
                  fontWeight: editProvider === p ? 700 : 400,
                }}
              >
                {p}
              </button>
            ))}
          </div>

          <label htmlFor="cfg-model" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Model</label>
          <input id="cfg-model" value={editModel} onChange={(e) => setEditModel(e.target.value)} style={inputStyle} />

          <label htmlFor="cfg-apikey" style={{ fontSize: 10, color: 'var(--text-dim)' }}>API Key</label>
          <input
            id="cfg-apikey"
            type="password"
            value={editApiKey}
            onChange={(e) => setEditApiKey(e.target.value)}
            placeholder="Leave blank to keep current"
            style={inputStyle}
          />

          <label htmlFor="cfg-baseurl" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Base URL</label>
          <input id="cfg-baseurl" value={editBaseUrl} onChange={(e) => setEditBaseUrl(e.target.value)} style={inputStyle} />

          <button
            onClick={handleTest}
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

          {testResult && (
            <div style={{
              fontSize: 10,
              padding: '4px 8px',
              borderRadius: 3,
              marginBottom: 8,
              background: testResult.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
              color: testResult.ok ? 'var(--accent-green)' : '#ef4444',
              border: `1px solid ${testResult.ok ? 'var(--accent-green)' : '#ef4444'}`,
            }}>
              {testResult.message}
            </div>
          )}

          {saveError && (
            <div style={{
              fontSize: 10,
              padding: '4px 8px',
              borderRadius: 3,
              marginBottom: 8,
              background: 'rgba(239,68,68,0.1)',
              color: '#ef4444',
              border: '1px solid #ef4444',
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
                 if (window.confirm('Revoke this invite? Anyone with the link will lose access.')) {
                   revoke_token(active_ring_id!, t.token)
                 }
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
                <div style={{ flex: 1, padding: 4, border: '1px solid var(--border)', borderRadius: 2, textAlign: 'center', fontSize: 9, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={() => { const note = window.prompt('Rejection reason (optional):'); reject_request(active_ring_id!, req.id, note || undefined) }}>REJECT</div>
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
              } catch (e: any) {
                const msg = typeof e?.message === 'string' ? e.message : typeof e === 'string' ? e : JSON.stringify(e)
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
    </div>
  )
}
