export function StepDone() {
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
          maxWidth: 360,
          margin: '0 auto',
          color: 'var(--text-secondary)',
          fontSize: 12,
          lineHeight: 2,
        }}
      >
        <div style={{ color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
          Quick Commands
        </div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>@self</span> — Open Self</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>#node</span> — Reference graph node</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!graph</span> — Open Graph panel</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!save</span> — Trigger archive</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!auto</span> — Toggle Auto mode</div>
      </div>
    </div>
  )
}
