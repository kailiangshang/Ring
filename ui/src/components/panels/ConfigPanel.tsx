import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)

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
  }, [active_ring_id])

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
        <div
          key={m.token_id}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: '4px 0',
            color: 'var(--text-primary)',
          }}
        >
          <span>{m.display_name}</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>({m.role})</span>
          {m.online && (
            <span style={{ color: 'var(--accent-green)', fontSize: 10 }}>●</span>
          )}
        </div>
      ))}
      {members.length === 0 && (
        <p style={{ color: 'var(--text-dim)' }}>No members</p>
      )}
    </div>
  )
}
