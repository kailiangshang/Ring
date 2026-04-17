export function SelfSettings() {
  return (
    <div style={{ padding: 8, fontSize: 12 }}>
      <div style={{ marginBottom: 12 }}>
        <label style={{ color: 'var(--text-dim)', fontSize: 10, letterSpacing: '0.05em' }}>
          Identity
        </label>
        <textarea
          style={{
            width: '100%',
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: 8,
            color: 'var(--text-primary)',
            fontSize: 12,
            fontFamily: 'inherit',
            resize: 'vertical',
            minHeight: 60,
            outline: 'none',
          }}
          defaultValue="I am your personal AI assistant"
        />
      </div>
      <div style={{ marginBottom: 12 }}>
        <label style={{ color: 'var(--text-dim)', fontSize: 10, letterSpacing: '0.05em' }}>
          Autonomy Level
        </label>
        <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
          {['suggest', 'assist', 'auto'].map((level) => (
            <button
              key={level}
              style={{
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '2px 8px',
                fontSize: 11,
                cursor: 'pointer',
              }}
            >
              {level}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
