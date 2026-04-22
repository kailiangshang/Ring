interface StepDoneProps {
  token: string | null
  onEnter: () => void
}

const sectionStyle: React.CSSProperties = {
  marginBottom: 12,
}

const sectionTitleStyle: React.CSSProperties = {
  color: 'var(--text-dim)',
  fontSize: 11,
  fontWeight: 700,
  letterSpacing: '0.05em',
  marginBottom: 6,
  textTransform: 'uppercase' as const,
}

const cmdStyle: React.CSSProperties = {
  color: 'var(--accent-cyan)',
  fontFamily: 'monospace',
  fontSize: 12,
}

export function StepDone({ onEnter }: StepDoneProps) {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>&#10003;</div>
      <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent-green)', marginBottom: 16 }}>
        Setup Complete
      </h1>

      <div
        style={{
          textAlign: 'left',
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: 16,
          maxWidth: 420,
          margin: '0 auto 20px',
          color: 'var(--text-secondary)',
          fontSize: 12,
          lineHeight: 1.8,
        }}
      >
        <div style={sectionStyle}>
          <div style={sectionTitleStyle}>Environment Commands (/)</div>
          <div><span style={cmdStyle}>/graph</span> — Open graph panel</div>
          <div><span style={cmdStyle}>/archive</span> — Open archive panel</div>
          <div><span style={cmdStyle}>/session</span> — Session operations</div>
          <div><span style={cmdStyle}>/save</span> — Archive conversation</div>
          <div><span style={cmdStyle}>{'/node add <name>'}</span> — Add graph node</div>
          <div><span style={cmdStyle}>/mode auto</span> — Toggle auto mode</div>
          <div><span style={cmdStyle}>/prefs</span> — Show/set preferences</div>
          <div><span style={cmdStyle}>/skill list</span> — Manage skills</div>
        </div>

        <div style={sectionStyle}>
          <div style={sectionTitleStyle}>Addressing (@)</div>
          <div><span style={cmdStyle}>{'@self <msg>'}</span> — Talk to Self</div>
          <div><span style={cmdStyle}>{'@ring <msg>'}</span> — Talk to Ring AI</div>
          <div><span style={cmdStyle}>{'@super <msg>'}</span> — Talk to Super Ring</div>
          <div><span style={cmdStyle}>{'@node <name>'}</span> — Reference node</div>
        </div>

        <div style={{ marginTop: 8, fontSize: 11, color: 'var(--text-dim)', fontStyle: 'italic' }}>
          Type /help for full command list
        </div>
      </div>

      <button
        onClick={onEnter}
        style={{
          background: 'var(--accent-cyan)',
          color: 'var(--bg-base)',
          border: 'none',
          borderRadius: 4,
          padding: '10px 32px',
          fontSize: 13,
          fontWeight: 700,
          cursor: 'pointer',
          letterSpacing: '0.05em',
        }}
      >
        Enter Ring
      </button>
    </div>
  )
}
