import { useCallback, useEffect, useState } from 'react'
import { api } from '../../services/api'
import { ConfirmModal } from '../common/ConfirmModal'

const FILE_LABELS: Record<string, string> = {
  user_profile: '用户画像',
  preferences: '偏好',
  active_goals: '当前目标',
  growth: '成长轨迹',
}

interface MemoryFile {
  name: string
  exists: boolean
  line_count: number
  size: number
}

export function SelfMemory() {
  const [files, setFiles] = useState<MemoryFile[]>([])
  const [editing, setEditing] = useState<string | null>(null)
  const [editContent, setEditContent] = useState('')
  const [saving, setSaving] = useState(false)
  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)
  const [saveWarning, setSaveWarning] = useState(false)

  const loadData = useCallback(async () => {
    try {
      const fileList = await api.get<MemoryFile[]>('/self/memory')
      setFiles(fileList)
    } catch {
      // silently ignore
    }
  }, [])

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      void loadData()
    }, 0)
    return () => window.clearTimeout(timeout)
  }, [loadData])

  const openEdit = async (name: string) => {
    try {
      const result = await api.get<{ content: string }>(`/self/memory/${name}`)
      setEditContent(result.content)
      setEditing(name)
    } catch {
      // silently ignore
    }
  }

  const saveEdit = async () => {
    if (!editing) return
    if (editContent.length > 2000) {
      setSaveWarning(true)
      return
    }
    doSave()
  }

  const doSave = async () => {
    if (!editing) return
    setSaveWarning(false)
    setSaving(true)
    try {
      await api.put(`/self/memory/${editing}`, { content: editContent })
      setEditing(null)
      loadData()
    } catch {
      // silently ignore
    } finally {
      setSaving(false)
    }
  }

  const deleteFile = async (name: string) => {
    try {
      await api.delete(`/self/memory/${name}`)
      loadData()
    } catch {
      // silently ignore
    }
  }

  const createFile = async () => {
    if (!newName.trim()) return
    const name = newName.trim().toLowerCase().replace(/\s+/g, '_')
    try {
      await api.put(`/self/memory/${name}`, { content: '' })
      setCreating(false)
      setNewName('')
      await loadData()
      openEdit(name)
    } catch {
      // silently ignore
    }
  }

  if (editing) {
    const label = FILE_LABELS[editing] || editing
    const lineCount = editContent.split('\n').filter(l => l.trim()).length
    return (
      <div style={{ padding: 12, height: '100%', display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
          <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-amber)' }}>{label}</span>
          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
              {lineCount} 行 · {editContent.length}B
            </span>
            <button onClick={() => { setEditing(null); setSaveWarning(false) }} style={{ ...btnStyle, color: 'var(--text-dim)' }}>取消</button>
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
        <ConfirmModal
          open={saveWarning}
          title="内容较长"
          message="内容较长，AI 会自动压缩"
          on_confirm={doSave}
          on_cancel={() => setSaveWarning(false)}
          confirm_label="仍然保存"
          cancel_label="取消"
        />
      </div>
    )
  }

  return (
    <div style={{ padding: 12, overflowY: 'auto', height: '100%' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
        <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em' }}>
          MEMORY FILES
        </span>
        <button
          onClick={() => setCreating(true)}
          style={{ background: 'none', border: '1px solid var(--border)', borderRadius: 3, color: 'var(--accent-amber)', fontSize: 10, padding: '2px 8px', cursor: 'pointer', fontWeight: 700 }}
        >
          + NEW
        </button>
      </div>

      {creating && (
        <div style={{ border: '1px solid var(--accent-amber)', borderRadius: 4, padding: '6px 10px', marginBottom: 6, background: 'var(--bg-active)', display: 'flex', gap: 6, alignItems: 'center' }}>
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => { if (e.key === 'Enter' && !e.nativeEvent.isComposing) createFile(); if (e.key === 'Escape') { setCreating(false); setNewName('') } }}
            placeholder="file_name (no .md)"
            autoFocus
            style={{ flex: 1, background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 3, padding: '4px 8px', color: 'var(--text-primary)', fontSize: 11, outline: 'none', fontFamily: 'inherit' }}
          />
          <button onClick={createFile} style={{ background: 'var(--accent-amber)', border: 'none', borderRadius: 3, color: 'var(--bg-base)', fontSize: 10, padding: '3px 8px', cursor: 'pointer', fontWeight: 700 }}>OK</button>
          <button onClick={() => { setCreating(false); setNewName('') }} style={{ background: 'none', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 10 }}>✕</button>
        </div>
      )}

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
                  onClick={(e) => { e.stopPropagation(); setDeleteTarget(f.name) }}
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

      <div style={{ marginTop: 12, fontSize: 10, color: 'var(--text-dim)', textAlign: 'center' }}>
        Memory 100% private, stored in ~/.ring/self/
      </div>

      <ConfirmModal
        open={deleteTarget !== null}
        title="删除记忆文件"
        message={`确定删除「${deleteTarget ? (FILE_LABELS[deleteTarget] || deleteTarget) : ''}」？此操作不可撤销。`}
        on_confirm={() => {
          if (deleteTarget) {
            deleteFile(deleteTarget)
            setDeleteTarget(null)
          }
        }}
        on_cancel={() => setDeleteTarget(null)}
        confirm_label="删除"
        cancel_label="取消"
        variant="danger"
      />
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
