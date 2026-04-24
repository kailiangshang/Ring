import { useEffect, useState } from 'react'
import { api } from '../../services/api'

const FILE_LABELS: Record<string, string> = {
  user_profile: '用户画像',
  preferences: '偏好',
  active_goals: '当前目标',
}

interface MemoryFile {
  name: string
  exists: boolean
  line_count: number
  size: number
}

interface ToolUsage {
  tools?: Record<string, number>
}

interface DwellTime {
  views?: Record<string, number>
  daily?: Record<string, Record<string, number>>
}

interface Metrics {
  chat_patterns?: {
    total_messages?: number
    self_messages?: number
    total_chars?: number
  }
  session_stats?: { total_sessions?: number }
  archive_patterns?: { total_archives?: number }
  ring_activity?: { total_rings?: number }
  tool_usage?: ToolUsage
  dwell_time?: DwellTime
}

export function SelfMemory() {
  const [files, setFiles] = useState<MemoryFile[]>([])
  const [metrics, setMetrics] = useState<Metrics>({})
  const [editing, setEditing] = useState<string | null>(null)
  const [editContent, setEditContent] = useState('')
  const [saving, setSaving] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      const fileList = await api.get<MemoryFile[]>('/self/memory')
      setFiles(fileList)
    } catch {}
    try {
      const m = await api.get<Metrics>('/self/metrics')
      setMetrics(m)
    } catch {}
  }

  const openEdit = async (name: string) => {
    try {
      const result = await api.get<{ content: string }>(`/self/memory/${name}`)
      setEditContent(result.content)
      setEditing(name)
    } catch {}
  }

  const saveEdit = async () => {
    if (!editing) return
    setSaving(true)
    try {
      await api.put(`/self/memory/${editing}`, { content: editContent })
      setEditing(null)
      loadData()
    } catch {} finally {
      setSaving(false)
    }
  }

  const deleteFile = async (name: string) => {
    try {
      await api.delete(`/self/memory/${name}`)
      loadData()
    } catch {}
  }

  if (editing) {
    const label = FILE_LABELS[editing] || editing
    return (
      <div style={{ padding: 12, height: '100%', display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-amber)' }}>{label}</span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button onClick={() => setEditing(null)} style={{ ...btnStyle, color: 'var(--text-dim)' }}>取消</button>
            <button onClick={saveEdit} disabled={saving} style={{ ...btnStyle, color: 'var(--accent-amber)' }}>{saving ? '保存中...' : '保存'}</button>
          </div>
        </div>
        <textarea
          value={editContent}
          onChange={(e) => setEditContent(e.target.value)}
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: 8,
            color: 'var(--text-primary)',
            fontSize: 12,
            lineHeight: 1.5,
            resize: 'none',
            fontFamily: 'inherit',
            outline: 'none',
          }}
          placeholder="AI 会自动从对话中提取记忆到这里..."
        />
      </div>
    )
  }

  return (
    <div style={{ padding: 12, overflowY: 'auto', height: '100%' }}>
      <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
        MEMORY FILES
      </div>
      {files.map((f) => (
        <div
          key={f.name}
          style={{
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '8px 10px',
            marginBottom: 6,
            cursor: 'pointer',
            background: 'var(--bg-active)',
          }}
          onClick={() => openEdit(f.name)}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)' }}>
              {FILE_LABELS[f.name] || f.name}
            </span>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
              {f.exists && (
                <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
                  {f.line_count} 行 · {f.size}B
                </span>
              )}
              {f.exists && (
                <button
                  onClick={(e) => { e.stopPropagation(); deleteFile(f.name) }}
                  style={{ background: 'none', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 10, padding: '0 4px' }}
                  title="删除"
                >
                  ✕
                </button>
              )}
            </div>
          </div>
          {!f.exists && (
            <span style={{ fontSize: 10, color: 'var(--text-dim)', marginTop: 2, display: 'block' }}>
              暂无记忆 — AI 会从对话中自动提取
            </span>
          )}
        </div>
      ))}

      <div style={{ marginTop: 12, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
        ACTIVITY
      </div>
      <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
        <div>消息: {metrics.chat_patterns?.total_messages ?? 0} (Self: {metrics.chat_patterns?.self_messages ?? 0})</div>
        <div>Ring: {metrics.ring_activity?.total_rings ?? 0} | Session: {metrics.session_stats?.total_sessions ?? 0} | 归档: {metrics.archive_patterns?.total_archives ?? 0}</div>
      </div>

      {metrics.tool_usage?.tools && Object.keys(metrics.tool_usage.tools).length > 0 && (
        <>
          <div style={{ marginTop: 10, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
            TOOLS
          </div>
          <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
            {Object.entries(metrics.tool_usage.tools)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 6)
              .map(([name, count]) => (
                <div key={name}>{name}: {count}</div>
              ))}
          </div>
        </>
      )}

      {metrics.dwell_time?.daily && (() => {
        const today = new Date().toISOString().slice(0, 10)
        const todayData = metrics.dwell_time.daily[today]
        if (!todayData || Object.keys(todayData).length === 0) return null
        const fmtMin = (s: number) => s > 0 ? `${Math.round(s / 60)}min` : '0min'
        return (
          <>
            <div style={{ marginTop: 10, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
              TODAY
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
              {todayData.self_panel && <div>Self: {fmtMin(todayData.self_panel)}</div>}
              {todayData.ring_chat && <div>聊天: {fmtMin(todayData.ring_chat)}</div>}
              {todayData.graph && <div>图谱: {fmtMin(todayData.graph)}</div>}
              {todayData.session && <div>Session: {fmtMin(todayData.session)}</div>}
              {todayData.archive && <div>归档: {fmtMin(todayData.archive)}</div>}
            </div>
          </>
        )
      })()}

      <div style={{ marginTop: 12, fontSize: 10, color: 'var(--text-dim)', textAlign: 'center' }}>
        Memory 100% private, stored in ~/.ring/self/
      </div>
    </div>
  )
}

const btnStyle: React.CSSProperties = {
  background: 'none',
  border: 'none',
  fontSize: 11,
  fontWeight: 700,
  cursor: 'pointer',
  padding: '2px 6px',
}
