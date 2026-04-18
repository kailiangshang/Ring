import { useSessionStore } from '../../stores/session-store'
import { usePanelStore } from '../../stores/panel-store'

export function SessionIndicator() {
  const session = useSessionStore((s) => s.active_session)
  const participants = useSessionStore((s) => s.participants)
  const toggle = usePanelStore((s) => s.toggle)

  return (
    <div
      onClick={() => toggle('session')}
      style={{
        marginLeft: 28,
        padding: '4px 8px',
        fontSize: 11,
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        cursor: 'pointer',
      }}
    >
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: '50%',
          background: 'var(--accent-green)',
          flexShrink: 0,
        }}
      />
      {session ? `${session.title} · ${participants.length}` : '1 active session'}
    </div>
  )
}
