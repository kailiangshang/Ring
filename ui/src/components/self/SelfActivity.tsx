import { useEffect, useState } from 'react'
import { api } from '../../services/api'

interface ToolUsage {
  tools?: Record<string, number>
}

interface DwellTime {
  daily?: Record<string, Record<string, number>>
}

interface Metrics {
  chat_patterns?: {
    total_messages?: number
    self_messages?: number
    total_chars?: number
    [key: string]: unknown
  }
  session_stats?: { total_sessions?: number }
  archive_patterns?: { total_archives?: number }
  ring_activity?: {
    total_rings?: number
    rings?: Array<{ id: string; name: string }>
  }
  tool_usage?: ToolUsage
  dwell_time?: DwellTime
}

function barWidth(count: number, max: number): number {
  if (max === 0) return 0
  return Math.max(2, Math.round((count / max) * 20))
}

function barChars(width: number): string {
  return '█'.repeat(width) + '░'.repeat(Math.max(0, 20 - width))
}

export function SelfActivity() {
  const [metrics, setMetrics] = useState<Metrics>({})

  useEffect(() => {
    api.get<Metrics>('/self/metrics')
      .then(setMetrics)
      .catch(() => {})
  }, [])

  const ringMessages: Array<{ name: string; count: number }> = []
  if (metrics.chat_patterns) {
    const cp = metrics.chat_patterns
    const ringMap = new Map<string, string>()
    if (metrics.ring_activity?.rings) {
      for (const r of metrics.ring_activity.rings) {
        ringMap.set(r.id, r.name)
      }
    }
    for (const [key, val] of Object.entries(cp)) {
      const ringId = key.startsWith('ring_') ? key.slice(5) : null
      if (ringId && typeof val === 'number') {
        ringMessages.push({ name: ringMap.get(ringId) || ringId, count: val })
      }
    }
  }
  ringMessages.sort((a, b) => b.count - a.count)
  const topRings = ringMessages.slice(0, 5)
  const maxRingCount = topRings.length > 0 ? topRings[0].count : 0

  const dailyCounts: Array<{ date: string; count: number }> = []
  if (metrics.chat_patterns) {
    const today = new Date()
    for (let i = 6; i >= 0; i--) {
      const d = new Date(today)
      d.setDate(d.getDate() - i)
      const dateStr = d.toISOString().slice(0, 10)
      const dailyKey = `daily_${dateStr}`
      const count = (metrics.chat_patterns[dailyKey] as number) ?? 0
      dailyCounts.push({ date: dateStr.slice(5), count })
    }
  }
  const maxDaily = Math.max(...dailyCounts.map(d => d.count), 1)

  const topTools: Array<{ name: string; count: number }> = []
  if (metrics.tool_usage?.tools) {
    for (const [name, count] of Object.entries(metrics.tool_usage.tools)) {
      topTools.push({ name, count })
    }
    topTools.sort((a, b) => b.count - a.count)
  }
  const top3Tools = topTools.slice(0, 3)

  const totalMessages = metrics.chat_patterns?.total_messages ?? 0
  const totalArchives = metrics.archive_patterns?.total_archives ?? 0
  const totalSessions = metrics.session_stats?.total_sessions ?? 0
  const totalRings = metrics.ring_activity?.total_rings ?? 0

  const sectionTitle: React.CSSProperties = {
    fontSize: 10,
    fontWeight: 700,
    color: 'var(--text-dim)',
    letterSpacing: '0.1em',
    marginBottom: 6,
    marginTop: 10,
  }

  return (
    <div style={{ padding: 10, overflowY: 'auto', height: '100%', fontSize: 11 }}>
      <div style={sectionTitle}>RING ACTIVITY</div>
      {topRings.length === 0 ? (
        <div style={{ color: 'var(--text-dim)', fontSize: 10 }}>暂无数据</div>
      ) : (
        topRings.map((r) => (
          <div key={r.name} style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 3 }}>
            <span style={{ width: 60, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: 10, color: 'var(--text-secondary)' }}>{r.name}</span>
            <span style={{ fontFamily: 'monospace', fontSize: 9, color: 'var(--accent-amber)', opacity: 0.6 }}>{barChars(barWidth(r.count, maxRingCount))}</span>
            <span style={{ fontSize: 10, color: 'var(--text-dim)', minWidth: 24, textAlign: 'right' }}>{r.count}</span>
          </div>
        ))
      )}

      <div style={sectionTitle}>7-DAY TREND</div>
      {dailyCounts.map((d) => (
        <div key={d.date} style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
          <span style={{ width: 36, fontSize: 10, color: 'var(--text-dim)' }}>{d.date}</span>
          <span style={{ fontFamily: 'monospace', fontSize: 9, color: 'var(--accent-cyan)', opacity: 0.5 }}>
            {'█'.repeat(Math.max(1, Math.round((d.count / maxDaily) * 16)))}
          </span>
          <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>{d.count}</span>
        </div>
      ))}

      {top3Tools.length > 0 && (
        <>
          <div style={sectionTitle}>TOP TOOLS</div>
          {top3Tools.map((t) => (
            <div key={t.name} style={{ fontSize: 10, color: 'var(--text-secondary)', marginBottom: 2 }}>
              {t.name}: {t.count}
            </div>
          ))}
        </>
      )}

      <div style={sectionTitle}>STATS</div>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
        {[
          { label: '消息', value: totalMessages },
          { label: 'Ring', value: totalRings },
          { label: '归档', value: totalArchives },
          { label: 'Session', value: totalSessions },
        ].map((s) => (
          <div key={s.label} style={{ background: 'var(--bg-base)', border: '1px solid var(--border)', borderRadius: 4, padding: '6px 8px', textAlign: 'center' }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: 'var(--accent-ice)' }}>{s.value}</div>
            <div style={{ fontSize: 9, color: 'var(--text-dim)', marginTop: 2 }}>{s.label}</div>
          </div>
        ))}
      </div>
    </div>
  )
}
