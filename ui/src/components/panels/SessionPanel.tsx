import { useEffect, useRef, useState } from 'react'
import { useSessionStore } from '../../stores/session-store'
import { useRingStore } from '../../stores/ring-store'
import { useWsStore } from '../../stores/ws-store'
import { exportSessionMessages, uploadFile } from '../../services/api'
import { ScrollContainer } from '../common/ScrollContainer'
import type { SessionSkill } from '../../types/session'
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
          if (e.key === 'Enter' && title.trim()) handleCreate()
        }}
        placeholder="Session title..."
        style={{
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 10px',
          color: 'var(--text-primary)',
          fontSize: 12,
          fontFamily: 'inherit',
          outline: 'none',
          width: '100%',
          boxSizing: 'border-box',
        }}
      />

      <textarea
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        placeholder="Description (optional)..."
        rows={2}
        style={{
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 10px',
          color: 'var(--text-primary)',
          fontSize: 11,
          fontFamily: 'inherit',
          outline: 'none',
          resize: 'vertical',
          width: '100%',
          boxSizing: 'border-box',
        }}
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

function MaterialPrepView() {
  const session = useSessionStore((s) => s.active_session)
  const materials = useSessionStore((s) => s.materials)
  const fetchMaterials = useSessionStore((s) => s.fetchMaterials)
  const highlightMaterial = useSessionStore((s) => s.highlightMaterial)
  const startSession = useSessionStore((s) => s.startSession)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)

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
      } catch (e: any) {
        console.error('upload failed:', e.message)
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

        {materials.length === 0 ? (
          <div style={{ padding: '16px 0', textAlign: 'center' }}>
            <div style={{ color: 'var(--text-dim)', fontSize: 11, marginBottom: 8 }}>
              No materials prepared yet.
            </div>
            <div style={{ color: 'var(--text-dim)', fontSize: 10 }}>
              Materials can be suggested by AI based on your session topic, or added manually during discussion.
            </div>
          </div>
        ) : (
          materials.map((mat) => (
            <div
              key={mat.id}
              style={{ padding: '8px 0', borderBottom: '1px solid var(--border)' }}
            >
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
                    onClick={() => handleHighlight(mat.id)}
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
          ))
        )}
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8 }}>
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
  const [summary, setSummary] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [started, setStarted] = useState(false)

  const triggerSummarize = () => {
    if (!session) return
    setStarted(true)

    const token = localStorage.getItem('ring_token')
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return

    const url = `/api/rings/${ring_id}/sessions/${session.id}/summarize`

    fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { 'X-Ring-Token': token } : {}),
      },
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
                if (currentEvent === 'delta' && parsed.content) {
                  setSummary((prev) => prev + parsed.content)
                }
                if (currentEvent === 'error') {
                  setError(parsed.error ?? 'Unknown error')
                }
                if (currentEvent === 'message_end') {
                  fetchActiveSession(ring_id)
                }
              } catch {
              }
              currentEvent = ''
            }
          }
        }
      })
      .catch((e) => setError(e.message))
  }

  if (!session) return null

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
          {session.title}
        </div>
        <div style={{ fontSize: 10, color: 'var(--accent-amber)' }}>
          {started ? 'Generating summary...' : 'Ready to summarize'}
        </div>
      </div>

      <ScrollContainer>
        {!started ? (
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
        ) : (
          <div style={{ color: 'var(--text-primary)', fontSize: 11, whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>
            {summary || (
              <span style={{ color: 'var(--text-dim)' }}>Waiting for AI response...</span>
            )}
            {summary && !error && (
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
  const sendMessage = useSessionStore((s) => s.sendMessage)
  const closeSession = useSessionStore((s) => s.closeSession)
  const reopenSession = useSessionStore((s) => s.reopenSession)
  const deleteSession = useSessionStore((s) => s.deleteSession)
  const toggleArchive = useSessionStore((s) => s.toggleArchive)
  const clearActive = useSessionStore((s) => s.clearActive)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const connected = useWsStore((s) => s.connected)

  const [input, setInput] = useState('')

  if (!session) return null

  const is_closed = session.phase === 'closed'
  const is_discussion = session.phase === 'discussion'
  const can_send = is_discussion && connected && !is_closed

  const handleSend = () => {
    if (!input.trim()) return
    sendMessage(session.id, input.trim())
    setInput('')
  }

  const handleClose = async () => {
    if (!active_ring_id) return
    if (!window.confirm('Close this session? Participants will no longer be able to send messages.')) return
    await closeSession(active_ring_id, session.id)
  }

  const handleReopen = async () => {
    if (!active_ring_id) return
    await reopenSession(active_ring_id, session.id)
  }

  const handleDelete = async () => {
    if (!active_ring_id) return
    if (!window.confirm('Delete this session permanently? All messages and materials will be lost.')) return
    await deleteSession(active_ring_id, session.id)
  }

  const handleArchive = async () => {
    if (!active_ring_id) return
    await toggleArchive(active_ring_id, session.id, !session.archive_enabled)
  }

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
            style={{ padding: '6px 0', borderBottom: '1px solid var(--border)' }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  color: msg.sender === session.owner ? 'var(--accent-ice)' : 'var(--accent-cyan)',
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
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8 }}>
        {can_send && (
          <div style={{ display: 'flex', gap: 6, marginBottom: 8 }}>
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
              }}
            />
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
          color: '#ef4444',
          border: '1px solid #ef4444',
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
