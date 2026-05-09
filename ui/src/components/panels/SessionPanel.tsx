import { useEffect, useRef, useState, memo } from 'react'
import { useSessionStore } from '../../stores/session-store'
import { useRingStore } from '../../stores/ring-store'
import { useWsStore } from '../../stores/ws-store'
import { exportSessionMessages, getToken, uploadFile } from '../../services/api'
import { ScrollContainer } from '../common/ScrollContainer'
import type { SessionSkill, SessionMaterial } from '../../types/session'
import { ConfirmModal } from '../common/ConfirmModal'
const PHASE_LABELS: Record<string, string> = {
  material_prep: 'Preparing Materials',
  discussion: 'In Discussion',
  summary: 'Generating Summary',
  closed: 'Closed',
}

const SKILLS: { value: SessionSkill; label: string }[] = [
  { value: 'discussion', label: 'Discussion' },
  { value: 'decision', label: 'Decision' },
  { value: 'research', label: 'Research' },
  { value: 'review', label: 'Review' },
  { value: 'retrospective', label: 'Retrospective' },
  { value: 'knowledge_sharing', label: 'Knowledge Sharing' },
]

const inputStyle: React.CSSProperties = {
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '6px 10px',
  color: 'var(--text-primary)',
  fontSize: 11,
  fontFamily: 'inherit',
  outline: 'none',
  width: '100%',
  boxSizing: 'border-box',
}

function CreateSessionForm() {
  const createSession = useSessionStore((s) => s.createSession)
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [skill, setSkill] = useState<SessionSkill>('discussion')
  const [archivable, setArchivable] = useState(false)
  const [creating, setCreating] = useState(false)

  const handleCreate = async () => {
    if (!title.trim()) return
    setCreating(true)
    await createSession({
      title: title.trim(),
      description: description.trim() || undefined,
      skill,
      archivable: archivable || undefined,
    })
    setCreating(false)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div style={{
        padding: '10px 12px',
        background: 'var(--bg-hover)',
        borderRadius: 4,
        borderLeft: '3px solid var(--accent-teal)',
      }}>
        <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 4 }}>
          Start a Session
        </div>
        <div style={{ fontSize: 10, color: 'var(--text-secondary)', lineHeight: 1.5 }}>
          Sessions are structured group discussions. Pick a Skill to set the format, add materials, then discuss and get an AI summary.
        </div>
      </div>

      <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', letterSpacing: '0.05em' }}>
        New Session
      </div>

      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.nativeEvent.isComposing && title.trim()) handleCreate()
        }}
        placeholder="Session title..."
        style={{ ...inputStyle, fontSize: 12 }}
      />

      <textarea
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        placeholder="Description (optional)..."
        rows={2}
        style={{ ...inputStyle, resize: 'vertical' }}
      />

      <div>
        <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
          Skill
        </div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          {SKILLS.map((s) => (
            <button
              key={s.value}
              onClick={() => setSkill(s.value)}
              style={{
                background: skill === s.value ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: skill === s.value ? 'var(--bg-base)' : 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              {s.label}
            </button>
          ))}
        </div>
      </div>

      <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11, color: 'var(--text-secondary)', cursor: 'pointer' }}>
        <input
          type="checkbox"
          checked={archivable}
          onChange={(e) => setArchivable(e.target.checked)}
          style={{ accentColor: 'var(--accent-cyan)' }}
        />
        Archive enabled
      </label>

      <button
        onClick={handleCreate}
        disabled={!title.trim() || creating}
        style={{
          background: title.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
          color: title.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
          border: 'none',
          borderRadius: 4,
          padding: '8px 16px',
          fontSize: 12,
          fontWeight: 700,
          cursor: title.trim() && !creating ? 'pointer' : 'default',
          letterSpacing: '0.05em',
          opacity: creating ? 0.6 : 1,
        }}
      >
        {creating ? 'Creating...' : 'CREATE'}
      </button>
    </div>
  )
}

const MaterialCard = memo(function MaterialCard({
  mat,
  onHighlight,
  onSave,
}: {
  mat: SessionMaterial
  onHighlight: (id: string) => void
  onSave: (id: string, title: string, content: string) => void
}) {
  const [editing, setEditing] = useState(false)
  const [editTitle, setEditTitle] = useState(mat.title)
  const [editContent, setEditContent] = useState(mat.content)

  const startEditing = () => {
    setEditTitle(mat.title)
    setEditContent(mat.content)
    setEditing(true)
  }

  const handleSave = () => {
    onSave(mat.id, editTitle.trim(), editContent.trim())
    setEditing(false)
  }

  if (editing) {
    return (
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
        <input
          value={editTitle}
          onChange={(e) => setEditTitle(e.target.value)}
          style={{ ...inputStyle, fontWeight: 700, marginBottom: 4 }}
        />
        <textarea
          value={editContent}
          onChange={(e) => setEditContent(e.target.value)}
          rows={3}
          style={{ ...inputStyle, resize: 'vertical' }}
        />
        <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
          <button
            onClick={handleSave}
            disabled={!editTitle.trim()}
            style={{
              background: 'var(--accent-cyan)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 3,
              padding: '3px 8px',
              fontSize: 10,
              fontWeight: 700,
              cursor: editTitle.trim() ? 'pointer' : 'default',
            }}
          >
            Save
          </button>
          <button
            onClick={() => setEditing(false)}
            style={{
              background: 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '3px 8px',
              fontSize: 10,
              color: 'var(--text-secondary)',
              cursor: 'pointer',
            }}
          >
            Cancel
          </button>
        </div>
      </div>
    )
  }

  return (
    <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)' }}>
          {mat.title}
        </span>
        <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
          <span
            style={{
              fontSize: 9,
              padding: '1px 6px',
              borderRadius: 2,
              background: mat.status === 'ready' ? 'var(--accent-green)' : mat.status === 'analyzing' ? 'var(--accent-amber)' : 'var(--bg-hover)',
              color: mat.status === 'ready' ? 'var(--bg-base)' : 'var(--text-dim)',
            }}
          >
            {mat.status}
          </span>
          <span
            style={{
              fontSize: 9,
              padding: '1px 4px',
              borderRadius: 2,
              background: 'var(--bg-hover)',
              color: 'var(--text-dim)',
            }}
          >
            {mat.item_type}
          </span>
          <button
            onClick={startEditing}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--text-dim)',
              cursor: 'pointer',
              fontSize: 10,
              padding: '0 2px',
            }}
          >
            Edit
          </button>
          <button
            onClick={() => onHighlight(mat.id)}
            style={{
              background: 'none',
              border: 'none',
              color: mat.highlight ? 'var(--accent-cyan)' : 'var(--text-dim)',
              cursor: 'pointer',
              fontSize: 10,
              padding: '0 2px',
            }}
          >
            ★
          </button>
        </div>
      </div>
      <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginTop: 2, lineHeight: 1.4 }}>
        {mat.content}
      </div>
      {mat.highlight && (
        <div style={{ fontSize: 10, color: 'var(--accent-cyan)', marginTop: 4, fontStyle: 'italic' }}>
          ★ {mat.highlight}
        </div>
      )}
    </div>
  )
})

function MaterialPrepView() {
  const session = useSessionStore((s) => s.active_session)
  const materials = useSessionStore((s) => s.materials)
  const fetchMaterials = useSessionStore((s) => s.fetchMaterials)
  const highlightMaterial = useSessionStore((s) => s.highlightMaterial)
  const updateMaterial = useSessionStore((s) => s.updateMaterial)
  const createMaterial = useSessionStore((s) => s.createMaterial)
  const startSession = useSessionStore((s) => s.startSession)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)
  const [showAddForm, setShowAddForm] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [newContent, setNewContent] = useState('')
  const [newType, setNewType] = useState('context')

  useEffect(() => {
    if (session && active_ring_id) {
      fetchMaterials(active_ring_id, session.id)
    }
  }, [session, active_ring_id, fetchMaterials])

  if (!session) return null

  const handleStart = async () => {
    if (!active_ring_id) return
    await startSession(active_ring_id, session.id)
  }

  const handleHighlight = async (material_id: string) => {
    if (!active_ring_id) return
    const note = prompt('Highlight note:')
    if (note) {
      await highlightMaterial(active_ring_id, session.id, material_id, note)
    }
  }

  const handleSaveMaterial = async (material_id: string, title: string, content: string) => {
    if (!active_ring_id) return
    await updateMaterial(active_ring_id, session.id, material_id, title, content)
  }

  const handleAddMaterial = async () => {
    if (!active_ring_id || !newTitle.trim()) return
    await createMaterial(active_ring_id, session.id, newType, newTitle.trim(), newContent.trim())
    setNewTitle('')
    setNewContent('')
    setNewType('context')
    setShowAddForm(false)
  }

  const handleUpload = async (files: FileList | null) => {
    if (!files || files.length === 0 || !active_ring_id) return
    setUploading(true)
    for (let i = 0; i < files.length; i++) {
      try {
        await uploadFile(
          `/rings/${active_ring_id}/sessions/${session.id}/material-prep/upload`,
          files[i],
        )
        fetchMaterials(active_ring_id, session.id)
      } catch (e: unknown) {
        console.error('upload failed:', e)
      }
    }
    setUploading(false)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
              {session.title}
            </div>
            <div style={{ fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8 }}>
              <span>Skill: {session.skill}</span>
              <span style={{ color: 'var(--accent-cyan)' }}>Phase: {PHASE_LABELS[session.phase] ?? session.phase}</span>
            </div>
          </div>
          <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
            <input
              ref={fileInputRef}
              type="file"
              multiple
              accept=".txt,.md,.csv,.json,.py,.js,.ts,.tsx,.rs,.go,.java,.yaml,.yml,.xml,.html,.css,.toml,.sh,.sql,.log,.pdf"
              style={{ display: 'none' }}
              onChange={(e) => handleUpload(e.target.files)}
            />
            <button
              onClick={() => fileInputRef.current?.click()}
              disabled={uploading}
              style={{
                background: 'var(--bg-active)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                color: 'var(--text-primary)',
                fontSize: 12,
                padding: '6px 12px',
                cursor: uploading ? 'default' : 'pointer',
              }}
            >
              {uploading ? 'Uploading...' : '📎 Upload Document'}
            </button>
          </div>
        </div>
      </div>

      <ScrollContainer>
        <div style={{
          padding: '12px',
          marginBottom: 8,
          background: 'var(--bg-hover)',
          borderRadius: 4,
          borderLeft: '3px solid var(--accent-cyan)',
        }}>
          <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 4 }}>
            Session created!
          </div>
          <div style={{ fontSize: 10, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
            This is the material preparation phase. Gather relevant documents, graph nodes, and context before starting the discussion. You can add highlight notes to materials, or skip directly to discussion.
          </div>
        </div>

        {materials.length === 0 && !showAddForm && (
          <div style={{ padding: '16px 0', textAlign: 'center' }}>
            <div style={{ color: 'var(--text-dim)', fontSize: 11, marginBottom: 8 }}>
              No materials prepared yet.
            </div>
            <div style={{ color: 'var(--text-dim)', fontSize: 10 }}>
              Materials can be suggested by AI based on your session topic, or added manually during discussion.
            </div>
          </div>
        )}
        {materials.map((mat) => (
          <MaterialCard
            key={mat.id}
            mat={mat}
            onHighlight={handleHighlight}
            onSave={handleSaveMaterial}
          />
        ))}

        {showAddForm && (
          <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
            <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
              Add Material
            </div>
            <div style={{ display: 'flex', gap: 4, marginBottom: 4 }}>
              {['context', 'question', 'data', 'reference'].map((t) => (
                <button
                  key={t}
                  onClick={() => setNewType(t)}
                  style={{
                    background: newType === t ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                    color: newType === t ? 'var(--bg-base)' : 'var(--text-dim)',
                    border: '1px solid var(--border)',
                    borderRadius: 2,
                    padding: '1px 6px',
                    fontSize: 9,
                    cursor: 'pointer',
                  }}
                >
                  {t}
                </button>
              ))}
            </div>
            <input
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              placeholder="Material title..."
              style={{ ...inputStyle, fontWeight: 700, marginBottom: 4 }}
            />
            <textarea
              value={newContent}
              onChange={(e) => setNewContent(e.target.value)}
              placeholder="Material content..."
              rows={3}
              style={{ ...inputStyle, resize: 'vertical' }}
            />
            <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
              <button
                onClick={handleAddMaterial}
                disabled={!newTitle.trim()}
                style={{
                  background: newTitle.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: newTitle.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
                  border: 'none',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  fontWeight: 700,
                  cursor: newTitle.trim() ? 'pointer' : 'default',
                }}
              >
                Add
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--text-secondary)',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8, display: 'flex', flexDirection: 'column', gap: 4 }}>
        {!showAddForm && (
          <button
            onClick={() => setShowAddForm(true)}
            style={{
              background: 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 12px',
              fontSize: 11,
              color: 'var(--text-secondary)',
              cursor: 'pointer',
              width: '100%',
            }}
          >
            + Add Material
          </button>
        )}
        <button
          onClick={handleStart}
          style={{
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            padding: '8px 16px',
            fontSize: 12,
            fontWeight: 700,
            cursor: 'pointer',
            width: '100%',
            letterSpacing: '0.05em',
          }}
        >
          START DISCUSSION
        </button>
      </div>
    </div>
  )
}

function SummarizeView() {
  const session = useSessionStore((s) => s.active_session)
  const fetchActiveSession = useSessionStore((s) => s.fetchActiveSession)
  const updateSummary = useSessionStore((s) => s.updateSummary)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [summary, setSummary] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [started, setStarted] = useState(false)
  const [editingSummary, setEditingSummary] = useState(false)
  const [editSummaryText, setEditSummaryText] = useState('')
  const abortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    return () => { abortRef.current?.abort() }
  }, [])

  const triggerSummarize = async () => {
    if (!session) return
    abortRef.current?.abort()
    const controller = new AbortController()
    abortRef.current = controller
    setStarted(true)

    const token = (await getToken()) ?? ''
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return

    const url = `/api/rings/${ring_id}/sessions/${session.id}/summarize`

    fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { 'X-Ring-Token': token } : {}),
      },
      signal: controller.signal,
    })
      .then(async (res) => {
        if (!res.ok) {
          const err = await res.json().catch(() => ({}))
          setError(err?.error?.message ?? 'Summarize failed')
          return
        }
        const reader = res.body?.getReader()
        if (!reader) { setError('No response body'); return }

        const decoder = new TextDecoder()
        let buffer = ''

        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          buffer += decoder.decode(value, { stream: true })
          const lines = buffer.split('\n')
          buffer = lines.pop() ?? ''

          let currentEvent = ''
          for (const line of lines) {
            if (line.startsWith('event: ')) {
              currentEvent = line.slice(7).trim()
            } else if (line.startsWith('data: ')) {
              const data = line.slice(6)
              try {
                const parsed = JSON.parse(data)
                if (currentEvent === 'delta' && typeof parsed.content === 'string') {
                  setSummary((prev) => prev + parsed.content)
                }
                if (currentEvent === 'error') {
                  setError(typeof parsed.error === 'string' ? parsed.error : 'Unknown error')
                }
                if (currentEvent === 'message_end') {
                  fetchActiveSession(ring_id)
                }
              } catch {
                // ignore parse errors
              }
              currentEvent = ''
            }
          }
        }
      })
      .catch((e) => setError(e.message))
  }

  if (!session) return null

  const is_complete = session.phase === 'closed' && session.summary
  const display_summary = is_complete ? (session.summary ?? '') : summary

  const handleSaveSummary = async () => {
    if (!active_ring_id || !session) return
    await updateSummary(active_ring_id, session.id, editSummaryText.trim())
    setEditingSummary(false)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
              {session.title}
            </div>
            <div style={{ fontSize: 10, color: is_complete ? 'var(--accent-green)' : 'var(--accent-amber)' }}>
              {is_complete ? 'Summary complete' : started ? 'Generating summary...' : 'Ready to summarize'}
            </div>
          </div>
          {is_complete && !editingSummary && (
            <button
              onClick={() => {
                setEditSummaryText(display_summary)
                setEditingSummary(true)
              }}
              style={{
                background: 'var(--bg-hover)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                color: 'var(--accent-cyan)',
                cursor: 'pointer',
              }}
            >
              Edit
            </button>
          )}
        </div>
      </div>

      <ScrollContainer>
        {!started && !is_complete ? (
          <div style={{ padding: 16, textAlign: 'center' }}>
            <button
              onClick={triggerSummarize}
              style={{
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 4,
                padding: '8px 16px',
                fontSize: 12,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              Generate Summary
            </button>
          </div>
        ) : error ? (
          <div style={{ padding: 16, color: 'var(--accent-amber)', fontSize: 12 }}>
            Error: {error}
          </div>
        ) : editingSummary ? (
          <div style={{ padding: '8px 0' }}>
            <textarea
              value={editSummaryText}
              onChange={(e) => setEditSummaryText(e.target.value)}
              rows={12}
              style={{ ...inputStyle, resize: 'vertical', lineHeight: 1.6, whiteSpace: 'pre-wrap' }}
            />
            <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
              <button
                onClick={handleSaveSummary}
                style={{
                  background: 'var(--accent-cyan)',
                  color: 'var(--bg-base)',
                  border: 'none',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  fontWeight: 700,
                  cursor: 'pointer',
                }}
              >
                Save
              </button>
              <button
                onClick={() => setEditingSummary(false)}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--text-secondary)',
                  cursor: 'pointer',
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        ) : (
          <div style={{ color: 'var(--text-primary)', fontSize: 11, whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>
            {display_summary || (
              <span style={{ color: 'var(--text-dim)' }}>Waiting for AI response...</span>
            )}
            {!is_complete && display_summary && !error && (
              <span style={{
                display: 'inline-block',
                width: 6,
                height: 14,
                background: 'var(--accent-cyan)',
                marginLeft: 2,
                verticalAlign: 'middle',
                animation: 'blink 1s step-end infinite',
              }} />
            )}
          </div>
        )}
      </ScrollContainer>
    </div>
  )
}

function SessionChat() {
  const session = useSessionStore((s) => s.active_session)
  const participants = useSessionStore((s) => s.participants)
  const messages = useSessionStore((s) => s.messages)
  const graph_suggestions = useSessionStore((s) => s.graph_suggestions)
  const sendMessage = useSessionStore((s) => s.sendMessage)
  const closeSession = useSessionStore((s) => s.closeSession)
  const reopenSession = useSessionStore((s) => s.reopenSession)
  const deleteSession = useSessionStore((s) => s.deleteSession)
  const toggleArchive = useSessionStore((s) => s.toggleArchive)
  const extractToGraph = useSessionStore((s) => s.extractToGraph)
  const clearActive = useSessionStore((s) => s.clearActive)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const connected = useWsStore((s) => s.connected)

  const [input, setInput] = useState('')
  const [confirmDialog, setConfirmDialog] = useState<{ title: string; message: string; action: () => void; variant?: 'danger' | 'default' } | null>(null)

  if (!session) return null

  const is_closed = session.phase === 'closed'
  const is_discussion = session.phase === 'discussion'
  const can_send = is_discussion && connected && !is_closed

  const handleSend = () => {
    if (!input.trim()) return
    sendMessage(session.id, input.trim())
    setInput('')
  }

  const handleClose = () => {
    if (!active_ring_id) return
    setConfirmDialog({
      title: 'Close Session',
      message: 'Close this session? Participants will no longer be able to send messages.',
      variant: 'danger',
      action: () => { closeSession(active_ring_id, session.id) },
    })
  }

  const handleReopen = async () => {
    if (!active_ring_id) return
    await reopenSession(active_ring_id, session.id)
  }

  const handleDelete = () => {
    if (!active_ring_id) return
    setConfirmDialog({
      title: 'Delete Session',
      message: 'Delete this session permanently? All messages and materials will be lost.',
      variant: 'danger',
      action: () => { deleteSession(active_ring_id, session.id) },
    })
  }

  const handleArchive = async () => {
    if (!active_ring_id) return
    await toggleArchive(active_ring_id, session.id, !session.archive_enabled)
  }

  const handleExtractToGraph = async () => {
    if (!active_ring_id) return
    await extractToGraph(active_ring_id, session.id)
  }

  const is_ai_message = (msg: typeof messages[0]) => msg.message_type === 'ai' || msg.sender === 'session-ai'

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
          {session.title}
        </div>
        <div style={{ fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <span>Skill: {session.skill}</span>
          <span style={{ color: is_closed ? 'var(--accent-amber)' : 'var(--accent-green)' }}>
            Phase: {PHASE_LABELS[session.phase] ?? session.phase}
          </span>
          <span>{participants.length} participants</span>
          {!connected && <span style={{ color: 'var(--accent-amber)' }}>disconnected</span>}
        </div>
      </div>

      <ScrollContainer autoScroll>
        {messages.map((msg) => (
          <div
            key={msg.id}
            style={{
              padding: '6px 0',
              borderBottom: '1px solid var(--border)',
              borderLeft: is_ai_message(msg) ? '2px solid var(--accent-teal)' : undefined,
              paddingLeft: is_ai_message(msg) ? 6 : undefined,
              background: is_ai_message(msg) ? 'rgba(6, 182, 212, 0.04)' : undefined,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  color: is_ai_message(msg)
                    ? 'var(--accent-teal)'
                    : msg.sender === session.owner ? 'var(--accent-ice)' : 'var(--accent-cyan)',
                  letterSpacing: '0.05em',
                }}
              >
                {msg.sender_name.toUpperCase()}
              </span>
              <span style={{ fontSize: 9, color: 'var(--text-dim)' }}>
                {new Date(msg.created_at).toLocaleTimeString()}
              </span>
            </div>
            <div style={{ color: 'var(--text-primary)', fontSize: 11, whiteSpace: 'pre-wrap', lineHeight: 1.5 }}>
              {msg.content}
            </div>
          </div>
        ))}
        {messages.length === 0 && (
          <div style={{ padding: '16px 0', color: 'var(--text-dim)', fontSize: 11, textAlign: 'center' }}>
            No messages yet
          </div>
        )}
        {is_closed && graph_suggestions.length > 0 && (
          <div style={{ marginTop: 8 }}>
            <div style={{ fontSize: 10, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
              Graph Suggestions
            </div>
            {graph_suggestions.map((suggestion, idx) => (
              <div
                key={idx}
                style={{
                  padding: '6px 8px',
                  marginBottom: 4,
                  background: 'var(--bg-hover)',
                  borderRadius: 4,
                  borderLeft: '2px solid var(--accent-teal)',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)' }}>
                    {suggestion.title}
                  </span>
                  <button
                    style={{
                      background: 'var(--accent-cyan)',
                      color: 'var(--bg-base)',
                      border: 'none',
                      borderRadius: 3,
                      padding: '2px 8px',
                      fontSize: 9,
                      fontWeight: 700,
                      cursor: 'pointer',
                    }}
                    onClick={() => {
                      if (active_ring_id) {
                        import('../../stores/graph-store').then(({ useGraphStore }) => {
                          useGraphStore.getState().createNode(active_ring_id, suggestion.title, 'concept')
                        })
                      }
                    }}
                  >
                    Add
                  </button>
                </div>
                <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginTop: 2, lineHeight: 1.4 }}>
                  {suggestion.content}
                </div>
              </div>
            ))}
          </div>
        )}
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8 }}>
        {can_send && (
          <div style={{ marginBottom: 4 }}>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  handleSend()
                }
              }}
              placeholder="message..."
              style={{
                flex: 1,
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '6px 10px',
                color: 'var(--text-primary)',
                fontSize: 11,
                fontFamily: 'inherit',
                outline: 'none',
                width: '100%',
                boxSizing: 'border-box',
              }}
            />
            <div style={{ display: 'flex', gap: 6, marginTop: 4 }}>
              <span style={{ fontSize: 9, color: 'var(--text-dim)', fontStyle: 'italic' }}>
                Mention @session-ai to get AI assistance
              </span>
              <div style={{ flex: 1 }} />
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                style={{
                  background: input.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: input.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
                  border: 'none',
                  borderRadius: 4,
                  padding: '6px 12px',
                  fontSize: 11,
                  fontWeight: 700,
                  cursor: input.trim() ? 'pointer' : 'default',
                }}
              >
                SEND
              </button>
            </div>
          </div>
        )}

        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
          {is_discussion && (
            <>
              <button
                onClick={handleClose}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                }}
              >
                Close
              </button>
              {session.skill !== 'discussion' && (
                <button
                  onClick={() => {
                    useSessionStore.setState((s) => ({
                      active_session: s.active_session ? { ...s.active_session, phase: 'summary' as const } : null,
                    }))
                  }}
                  style={{
                    background: 'var(--bg-hover)',
                    border: '1px solid var(--border)',
                    borderRadius: 3,
                    padding: '3px 8px',
                    fontSize: 10,
                    color: 'var(--accent-cyan)',
                    cursor: 'pointer',
                  }}
                >
                  Summarize
                </button>
              )}
            </>
          )}
          {is_closed && (
            <>
              <button
                onClick={handleReopen}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--accent-green)',
                  cursor: 'pointer',
                }}
              >
                Reopen
              </button>
              <button
                onClick={handleDelete}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                }}
              >
                Delete
              </button>
              {session.summary && (
                <button
                  onClick={handleExtractToGraph}
                  style={{
                    background: 'var(--bg-hover)',
                    border: '1px solid var(--border)',
                    borderRadius: 3,
                    padding: '3px 8px',
                    fontSize: 10,
                    color: 'var(--accent-teal)',
                    cursor: 'pointer',
                  }}
                >
                  Extract to Graph
                </button>
              )}
            </>
          )}
          {session.archivable && (
            <button
              onClick={handleArchive}
              style={{
                background: 'var(--bg-hover)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                color: session.archive_enabled ? 'var(--accent-cyan)' : 'var(--text-dim)',
                cursor: 'pointer',
              }}
            >
              {session.archive_enabled ? 'Archive: ON' : 'Archive: OFF'}
            </button>
          )}
          <button
            onClick={() => active_ring_id && exportSessionMessages(active_ring_id, session.id)}
            style={{
              background: 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '3px 8px',
              fontSize: 10,
              color: 'var(--text-secondary)',
              cursor: 'pointer',
            }}
          >
            Export
          </button>
          <button
            onClick={clearActive}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--text-dim)',
              cursor: 'pointer',
              fontSize: 10,
              marginLeft: 'auto',
            }}
          >
            Leave session
          </button>
        </div>
      </div>
      <ConfirmModal
        open={confirmDialog !== null}
        title={confirmDialog?.title ?? ''}
        message={confirmDialog?.message ?? ''}
        variant={confirmDialog?.variant}
        on_confirm={() => { confirmDialog?.action(); setConfirmDialog(null) }}
        on_cancel={() => setConfirmDialog(null)}
      />
    </div>
  )
}

export function SessionPanel() {
  const active_session = useSessionStore((s) => s.active_session)
  const loading = useSessionStore((s) => s.loading)
  const error = useSessionStore((s) => s.error)
  const fetchActiveSession = useSessionStore((s) => s.fetchActiveSession)
  const handleWsMessage = useSessionStore((s) => s.handleWsMessage)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const connected = useWsStore((s) => s.connected)
  const wsConnect = useWsStore((s) => s.connect)
  const addHandler = useWsStore((s) => s.addHandler)
  const removeHandler = useWsStore((s) => s.removeHandler)

  useEffect(() => {
    wsConnect()
  }, [wsConnect])

  useEffect(() => {
    addHandler(handleWsMessage)
    return () => removeHandler(handleWsMessage)
  }, [addHandler, removeHandler, handleWsMessage])

  useEffect(() => {
    if (active_ring_id && !active_session) {
      fetchActiveSession(active_ring_id)
    }
  }, [active_ring_id, active_session, fetchActiveSession])

  if (loading) {
    return (
      <div style={{ padding: 16, color: 'var(--text-dim)', fontSize: 12 }}>
        Loading session...
      </div>
    )
  }

  if (active_session) {
    if (active_session.phase === 'material_prep') return <MaterialPrepView />
    if (active_session.phase === 'summary') return <SummarizeView />
    return <SessionChat />
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      {error && (
        <div style={{
          padding: '6px 8px',
          marginBottom: 4,
          fontSize: 10,
          borderRadius: 3,
          background: 'rgba(239,68,68,0.1)',
          color: 'var(--accent-red, #ef4444)',
          border: '1px solid var(--accent-red, #ef4444)',
        }}>
          {error}
        </div>
      )}
      <CreateSessionForm />
      {!connected && (
        <div style={{ marginTop: 8, fontSize: 10, color: 'var(--accent-amber)' }}>
          WebSocket disconnected — messages may be delayed
        </div>
      )}
    </div>
  )
}
