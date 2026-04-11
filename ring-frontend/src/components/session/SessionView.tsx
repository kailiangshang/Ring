import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useSessionStore } from '../../stores/sessionStore'
import { useSessionChatStore } from '../../stores/sessionChatStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { Button } from '../../components/ui/Button'
import { Badge } from '../../components/ui/Badge'
import { Input } from '../../components/ui/Input'
import { EmptyState } from '../../components/ui/EmptyState'
import './SessionView.css'

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

  if (sessionId) {
    return (
      <div className="session-view">
        <div className="session-back">
          <Button variant="ghost" onClick={() => navigate(`/ring/${ringId}/sessions`)}>
            &larr; Back to Sessions
          </Button>
        </div>
        <div className="session-chat">
          <div className="session-chat-messages">
            {messages.map((msg) => (
              <ChatBubble key={msg.id} role={msg.role as 'user' | 'assistant'} content={msg.content} />
            ))}
            {is_streaming && <div className="session-chat-typing">AI is typing...</div>}
          </div>
          {chat_error && <p className="setup-error" role="alert">{chat_error}</p>}
          <div className="session-chat-input">
            <ChatInput on_send={handle_send} disabled={is_streaming} />
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="session-view">
      <div className="session-view-header">
        <h2 className="session-view-title">Sessions</h2>
        <Button onClick={() => set_show_create(!show_create)}>
          {show_create ? 'Cancel' : 'New Session'}
        </Button>
      </div>

      {show_create && (
        <div className="session-create-form">
          <Input
            placeholder="Session title (optional)"
            value={title}
            onChange={(e) => set_title(e.target.value)}
          />
          <Input
            input_type="select"
            value={scenario}
            onChange={(e) => set_scenario(e.target.value)}
          >
            {SCENARIOS.map((s) => (
              <option key={s.value} value={s.value}>{s.label}</option>
            ))}
          </Input>
          <Button onClick={handle_create}>Create</Button>
        </div>
      )}

      {error && <p className="setup-error" role="alert">{error}</p>}
      {loading && <p>Loading...</p>}
      {!loading && sessions.length === 0 && (
        <EmptyState
          icon="💬"
          title="No sessions yet"
          description="Start a new session to collaborate with AI."
        />
      )}

      {!loading && sessions.map((s) => (
        <div
          key={s.id}
          className="session-card"
          onClick={() => navigate(`/ring/${ringId}/sessions/${s.id}`)}
        >
          <Badge status={s.status === 'active' ? 'active' : 'closed'}>{s.status}</Badge>
          <div className="session-card-info">
            <div className="session-card-title">{s.title || 'Untitled Session'}</div>
            <div className="session-card-meta">
              {s.member_count} members
            </div>
          </div>
          {s.archive_enabled && <Badge variant="accent">Archive</Badge>}
          <div className="session-card-actions" onClick={(e) => e.stopPropagation()}>
            {s.status === 'active' && ringId && (
              <Button size="sm" variant="secondary" onClick={() => close_session(ringId, s.id)}>
                Close
              </Button>
            )}
            {ringId && (
              <Button size="sm" variant={s.archive_enabled ? 'primary' : 'secondary'} onClick={() => toggle_archive(ringId, s.id, !s.archive_enabled)}>
                {s.archive_enabled ? 'Archive On' : 'Auto Archive'}
              </Button>
            )}
            {ringId && (
              <Button size="sm" variant="danger" onClick={() => { if (confirm('Delete this session?')) delete_session(ringId, s.id) }}>
                Delete
              </Button>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
