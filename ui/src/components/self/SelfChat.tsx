export function SelfChat() {
  return (
    <div style={{ padding: 8, flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div style={{ flex: 1, color: 'var(--text-muted)', fontSize: 12, textAlign: 'center', paddingTop: 40 }}>
        Chat with Self...
      </div>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '6px 10px',
            color: 'var(--text-primary)',
            fontSize: 12,
            fontFamily: 'inherit',
            outline: 'none',
          }}
          placeholder="Chat with Self..."
        />
      </div>
    </div>
  )
}
