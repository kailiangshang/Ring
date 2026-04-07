import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useSessionStore } from '../../stores/sessionStore'
import { useSessionChatStore } from '../../stores/sessionChatStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'

const SCENARIOS = [
  { value: 'discussion', label: 'Discussion' },
  { value: 'deep_research', label: 'Deep Research' },
  { value: 'meeting_archive', label: 'Meeting Archive' },
  { value: 'learning_center', label: 'Learning Center' },
]

export function SessionView() {
  const { ringId, sessionId } = useParams<{ ringId: string; sessionId?: string }>()
  const navigate = useNavigate()
  const {
    sessions,
    loading,
    error,
    load_sessions,
    create_session,
    close_session,
    delete_session,
    toggle_archive,
  } = useSessionStore()
  const {
    messages,
    is_streaming,
    error: chat_error,
    load_history,
    send_message,
    reset: reset_chat,
  } = useSessionChatStore()
  const [title, set_title] = useState('')
  const [scenario, set_scenario] = useState('discussion')
  const [show_create, set_show_create] = useState(false)

  useEffect(() => {
    if (ringId && !sessionId) load_sessions(ringId)
  }, [ringId, sessionId, load_sessions])

  useEffect(() => {
    if (ringId && sessionId) {
      reset_chat()
      load_history(ringId, sessionId)
    }
  }, [ringId, sessionId, load_history, reset_chat])

  const handle_create = async () => {
    if (!ringId) return
    await create_session(ringId, { title: title || undefined, scenario })
    set_show_create(false)
    set_title('')
  }

  const handle_send = (content: string) => {
    if (!ringId || !sessionId) return
    send_message(ringId, sessionId, content)
  }

  const status_badge_color = (status: string) => {
    switch (status) {
      case 'active':
        return '#28a745'
      case 'closed':
        return '#888'
      default:
        return '#888'
    }
  }

  return (
    <div style={{ padding: '1.5rem', maxWidth: '800px', margin: '0 auto' }}>
      {sessionId && (
        <button
          onClick={() => navigate(`/ring/${ringId}/sessions`)}
          style={{ marginBottom: '1rem', padding: '0.3rem 0.8rem', cursor: 'pointer' }}
        >
          &larr; Back to Sessions
        </button>
      )}

      {!sessionId && (
        <>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
            <h2>Sessions</h2>
            <button
              onClick={() => set_show_create(!show_create)}
              style={{
                padding: '0.5rem 1rem',
                background: '#0366d6',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {show_create ? 'Cancel' : 'New Session'}
            </button>
          </div>

          {show_create && (
            <div style={{ padding: '1rem', border: '1px solid #ddd', borderRadius: '4px', marginBottom: '1rem' }}>
              <input
                placeholder="Session title (optional)"
                value={title}
                onChange={(e) => set_title(e.target.value)}
                style={{ width: '100%', padding: '0.5rem', marginBottom: '0.5rem', border: '1px solid #ddd', borderRadius: '4px' }}
              />
              <select
                value={scenario}
                onChange={(e) => set_scenario(e.target.value)}
                style={{ width: '100%', padding: '0.5rem', marginBottom: '0.5rem', border: '1px solid #ddd', borderRadius: '4px' }}
              >
                {SCENARIOS.map((s) => (
                  <option key={s.value} value={s.value}>{s.label}</option>
                ))}
              </select>
              <button
                onClick={handle_create}
                style={{ padding: '0.5rem 1rem', background: '#28a745', color: '#fff', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
              >
                Create
              </button>
            </div>
          )}

          {error && <p style={{ color: 'red' }}>{error}</p>}
          {loading && <p>Loading...</p>}
          {!loading && sessions.length === 0 && <p style={{ color: '#888' }}>No sessions yet</p>}

          {!loading && sessions.map((s) => (
            <div
              key={s.id}
              style={{ padding: '1rem', border: '1px solid #e0e0e0', borderRadius: '4px', marginBottom: '0.5rem', cursor: 'pointer' }}
              onClick={() => navigate(`/ring/${ringId}/sessions/${s.id}`)}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.5rem' }}>
                <span style={{ padding: '2px 8px', borderRadius: '3px', fontSize: '0.75rem', fontWeight: 600, color: '#fff', background: status_badge_color(s.status) }}>
                  {s.status}
                </span>
                <span style={{ fontWeight: 500 }}>{s.title || s.scenario}</span>
                <span style={{ color: '#888', fontSize: '0.85rem' }}>
                  {s.member_count ?? s.members?.length ?? 0} members
                </span>
                {s.archive_enabled && (
                  <span style={{ padding: '2px 6px', borderRadius: '3px', fontSize: '0.7rem', background: '#0366d6', color: '#fff' }}>
                    Archive
                  </span>
                )}
              </div>
              <div style={{ display: 'flex', gap: '0.5rem' }} onClick={(e) => e.stopPropagation()}>
                {s.status === 'active' && ringId && (
                  <button onClick={() => close_session(ringId, s.id)} style={{ padding: '0.3rem 0.6rem', background: '#ffc107', border: 'none', borderRadius: '3px', cursor: 'pointer', fontSize: '0.8rem' }}>
                    Close
                  </button>
                )}
                {ringId && (
                  <button onClick={() => toggle_archive(ringId, s.id, !s.archive_enabled)} style={{ padding: '0.3rem 0.6rem', background: s.archive_enabled ? '#0366d6' : '#28a745', color: '#fff', border: 'none', borderRadius: '3px', cursor: 'pointer', fontSize: '0.8rem' }}>
                    {s.archive_enabled ? 'Archive On' : 'Auto Archive'}
                  </button>
                )}
                {ringId && (
                  <button onClick={() => { if (confirm('Delete this session?')) delete_session(ringId, s.id) }} style={{ padding: '0.3rem 0.6rem', background: '#dc3545', color: '#fff', border: 'none', borderRadius: '3px', cursor: 'pointer', fontSize: '0.8rem' }}>
                    Delete
                  </button>
                )}
              </div>
            </div>
          ))}
        </>
      )}

      {sessionId && (
        <div style={{ display: 'flex', flexDirection: 'column', height: '70vh' }}>
          <div style={{ flex: 1, overflow: 'auto', padding: 16 }}>
            {messages.map((msg) => (
              <ChatBubble key={msg.id} role={msg.role as 'user' | 'assistant'} content={msg.content} />
            ))}
            {is_streaming && <div style={{ color: '#888', marginBottom: 8 }}>AI is typing...</div>}
          </div>
          {chat_error && <p style={{ color: 'red' }}>{chat_error}</p>}
          <div style={{ padding: 16, borderTop: '1px solid #eee' }}>
            <ChatInput on_send={handle_send} disabled={is_streaming} />
          </div>
        </div>
      )}
    </div>
  )
}
