import { useEffect, useState } from 'react'
import { api } from '../../services/api'

interface Metrics {
  session_stats: Record<string, unknown> | null
  tool_usage: Record<string, unknown> | null
  dwell_time: Record<string, unknown> | null
  archive_patterns: Record<string, unknown> | null
}

const sectionStyle: React.CSSProperties = {
  marginBottom: 12,
  padding: '6px 8px',
  background: 'var(--bg-base)',
  borderRadius: 3,
  border: '1px solid var(--border)',
}

const labelStyle: React.CSSProperties = {
  fontSize: 10,
  fontWeight: 700,
  color: 'var(--accent-amber)',
  letterSpacing: '0.05em',
  marginBottom: 4,
  display: 'block',
}

const valueStyle: React.CSSProperties = {
  fontSize: 11,
  color: 'var(--text-primary)',
  lineHeight: 1.6,
}

export function SelfMemory() {
  const [metrics, setMetrics] = useState<Metrics | null>(null)
  const [identity, setIdentity] = useState('')

  useEffect(() => {
    api.get<{ content: string; exists: boolean }>('/self/identity')
      .then((res) => { if (res.exists) setIdentity(res.content) })
      .catch(() => {})
    api.get<Metrics>('/self/metrics')
      .then((res) => setMetrics(res))
      .catch(() => {})
  }, [])

  const stats = metrics?.session_stats as Record<string, number> | null
  const totalSessions = stats?.total_sessions ?? 0
  const totalMessages = stats?.total_messages ?? 0
  const avgLength = stats?.avg_message_length ?? 0

  return (
    <div style={{ padding: 8, fontSize: 12, overflow: 'auto', flex: 1 }}>
      <div style={sectionStyle}>
        <span style={labelStyle}>Behavior Profile</span>
        <div style={valueStyle}>
          <div>Total sessions: {totalSessions}</div>
          <div>Total messages: {totalMessages}</div>
          <div>Avg message length: {avgLength > 0 ? Math.round(avgLength) : '-'} chars</div>
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Interaction Stats</span>
        <div style={valueStyle}>
          {metrics?.archive_patterns ? (
            Object.entries(metrics.archive_patterns).map(([k, v]) => (
              <div key={k}>{k}: {String(v)}</div>
            ))
          ) : (
            <div style={{ color: 'var(--text-dim)' }}>No data yet</div>
          )}
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Identity</span>
        <div style={{ ...valueStyle, whiteSpace: 'pre-wrap' }}>
          {identity || <span style={{ color: 'var(--text-dim)' }}>Not set</span>}
        </div>
      </div>

      <div style={{ fontSize: 9, color: 'var(--text-dim)', textAlign: 'center', marginTop: 8 }}>
        Memory 100% private, stored in ~/.ring/self/
      </div>
    </div>
  )
}
