import { MOCK_SESSION } from '../../services/mock-data'

export function SessionPanel() {
  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ color: 'var(--accent-ice)', fontWeight: 700, marginBottom: 8 }}>
        {MOCK_SESSION.title}
      </p>
      <p style={{ color: 'var(--text-muted)', marginBottom: 4 }}>
        Skill: {MOCK_SESSION.skill} &middot; Phase: {MOCK_SESSION.phase}
      </p>
      <p style={{ color: 'var(--text-secondary)' }}>{MOCK_SESSION.description}</p>
    </div>
  )
}
