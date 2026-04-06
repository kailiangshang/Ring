import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useSessionStore } from '../../stores/sessionStore'

const SCENARIOS = [
  { value: 'discussion', label: 'Discussion' },
  { value: 'deep_research', label: 'Deep Research' },
  { value: 'meeting_archive', label: 'Meeting Archive' },
  { value: 'learning_center', label: 'Learning Center' },
]

export function SessionView() {
  const { ringId } = useParams<{ ringId: string }>()
  const {
    sessions,
    loading,
    error,
    load_sessions,
    create_session,
    close_session,
    delete_session,
  } = useSessionStore()

  const [title, set_title] = useState('')
  const [scenario, set_scenario] = useState('discussion')
  const [show_create, set_show_create] = useState(false)

  useEffect(() => {
    if (ringId) load_sessions(ringId)
  }, [ringId, load_sessions])

  const handle_create = async () => {
    if (!ringId) return
    await create_session(ringId, {
      title: title || undefined,
      scenario,
    })
    set_show_create(false)
    set_title('')
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
        <div
          style={{
            padding: '1rem',
            border: '1px solid #ddd',
            borderRadius: '4px',
            marginBottom: '1rem',
          }}
        >
          <input
            placeholder="Session title (optional)"
            value={title}
            onChange={(e) => set_title(e.target.value)}
            style={{
              width: '100%',
              padding: '0.5rem',
              marginBottom: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
            }}
          />
          <select
            value={scenario}
            onChange={(e) => set_scenario(e.target.value)}
            style={{
              width: '100%',
              padding: '0.5rem',
              marginBottom: '0.5rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
            }}
          >
            {SCENARIOS.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </select>
          <button
            onClick={handle_create}
            style={{
              padding: '0.5rem 1rem',
              background: '#28a745',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Create
          </button>
        </div>
      )}

      {error && <p style={{ color: 'red' }}>{error}</p>}
      {loading && <p>Loading...</p>}

      {!loading && sessions.length === 0 && (
        <p style={{ color: '#888' }}>No sessions yet</p>
      )}

      {!loading &&
        sessions.map((s) => (
          <div
            key={s.id}
            style={{
              padding: '1rem',
              border: '1px solid #e0e0e0',
              borderRadius: '4px',
              marginBottom: '0.5rem',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', marginBottom: '0.5rem' }}>
              <span
                style={{
                  padding: '2px 8px',
                  borderRadius: '3px',
                  fontSize: '0.75rem',
                  fontWeight: 600,
                  color: '#fff',
                  background: status_badge_color(s.status),
                }}
              >
                {s.status}
              </span>
              <span style={{ fontWeight: 500 }}>
                {s.title || s.scenario}
              </span>
              <span style={{ color: '#888', fontSize: '0.85rem' }}>
                {s.members.length} members
              </span>
            </div>
            <div style={{ display: 'flex', gap: '0.5rem' }}>
              {s.status === 'active' && ringId && (
                <button
                  onClick={() => close_session(ringId, s.id)}
                  style={{
                    padding: '0.3rem 0.6rem',
                    background: '#ffc107',
                    border: 'none',
                    borderRadius: '3px',
                    cursor: 'pointer',
                    fontSize: '0.8rem',
                  }}
                >
                  Close
                </button>
              )}
              {ringId && (
                <button
                  onClick={() => {
                    if (confirm('Delete this session?'))
                      delete_session(ringId, s.id)
                  }}
                  style={{
                    padding: '0.3rem 0.6rem',
                    background: '#dc3545',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '3px',
                    cursor: 'pointer',
                    fontSize: '0.8rem',
                  }}
                >
                  Delete
                </button>
              )}
            </div>
          </div>
        ))}
    </div>
  )
}
