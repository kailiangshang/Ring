import { useEffect, useState } from 'react'
import { api } from '../../services/api'

interface Metrics {
  session_stats: Record<string, unknown> | null
  tool_usage: Record<string, unknown> | null
  dwell_time: Record<string, unknown> | null
  archive_patterns: Record<string, unknown> | null
  chat_patterns: Record<string, unknown> | null
  ring_activity: Record<string, unknown> | null
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

  const chatStats = metrics?.chat_patterns as Record<string, number> | null
  const totalMessages = chatStats?.total_messages ?? 0
  const avgLength = chatStats?.avg_message_length ?? 0

  const sessionStats = metrics?.session_stats as Record<string, number> | null
  const totalSessions = sessionStats?.total_sessions ?? 0

  const archiveStats = metrics?.archive_patterns as Record<string, unknown> | null

  const ringActivity = metrics?.ring_activity as Record<string, unknown> | null
  const totalRings = ringActivity?.total_rings as number ?? 0
  const rings = ringActivity?.rings as Array<{ id: string; name: string; joined_at: string }> | undefined

  return (
    <div style={{ padding: 8, fontSize: 12, overflow: 'auto', flex: 1 }}>
      <div style={sectionStyle}>
        <span style={labelStyle}>Chat Stats</span>
        <div style={valueStyle}>
          <div>Total messages: {totalMessages}</div>
          <div>Avg message length: {avgLength > 0 ? Math.round(avgLength) : '-'} chars</div>
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Session Stats</span>
        <div style={valueStyle}>
          <div>Total sessions: {totalSessions}</div>
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Archive Patterns</span>
        <div style={valueStyle}>
          {archiveStats?.total_archives !== undefined ? (
            <div>Total archives: {String(archiveStats.total_archives)}</div>
          ) : (
            <div style={{ color: 'var(--text-dim)' }}>No data yet</div>
          )}
          {archiveStats?.last_archive_date ? (
            <div>Last: {String(archiveStats.last_archive_date)}</div>
          ) : null}
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Ring Activity</span>
        <div style={valueStyle}>
          <div>Total rings: {totalRings}</div>
          {rings && rings.length > 0 && (
            <div style={{ marginTop: 4 }}>
              {rings.slice(0, 5).map((r) => (
                <div key={r.id} style={{ fontSize: 10, color: 'var(--text-secondary)' }}>
                  • {r.name}
                </div>
              ))}
            </div>
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
