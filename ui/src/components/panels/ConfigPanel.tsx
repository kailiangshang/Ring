import { MOCK_MEMBERS } from '../../services/mock-data'

export function ConfigPanel() {
  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        Members
      </p>
      {MOCK_MEMBERS.map((m) => (
        <div
          key={m.token_id}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: '4px 0',
            color: 'var(--text-primary)',
          }}
        >
          <span>{m.display_name}</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>({m.role})</span>
          {m.online && (
            <span style={{ color: 'var(--accent-green)', fontSize: 10 }}>&#9679;</span>
          )}
        </div>
      ))}
    </div>
  )
}
