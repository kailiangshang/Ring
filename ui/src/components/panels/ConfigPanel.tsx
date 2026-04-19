import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useInviteStore } from '../../stores/invite-store'

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)

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

  useEffect(() => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => setLlmConfig({ ...res, provider: res.provider as LLMProvider }))
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

  const time_remaining = (expires_at: string) => {
    const diff = new Date(expires_at).getTime() - Date.now()
    if (diff <= 0) return 'expired'
    const hours = Math.floor(diff / 3600000)
    if (hours > 24) return `${Math.floor(hours / 24)}d left`
    return `${hours}h left`
  }

  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        LLM Config
      </p>
      {llmConfig && (
        <div style={{ marginBottom: 16, color: 'var(--text-primary)', lineHeight: 1.8 }}>
          <div>Provider: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.provider}</span></div>
          <div>Model: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.model}</span></div>
          <div>API Key: {llmConfig.api_key_set ? '✓' : '✗'}</div>
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
              <span style={{ color: 'var(--text-dim)', fontSize: 9, cursor: 'pointer' }} onClick={() => revoke_token(active_ring_id!, t.token)}>revoke</span>
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
    </div>
  )
}
