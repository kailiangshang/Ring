interface StepProps {
  onNext: () => void
  onJoin: () => void
}

export function StepWelcome({ onNext, onJoin }: StepProps) {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>
        <img src="/logo-pixel.svg" alt="Ring" width="48" height="48" />
      </div>
      <h1 style={{ fontSize: 24, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 8 }}>
        Welcome to Ring
      </h1>
      <p style={{ color: 'var(--text-secondary)', marginBottom: 32, maxWidth: 400, margin: '0 auto 32px' }}>
        Group Knowledge Collaboration Space
      </p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, maxWidth: 240, margin: '0 auto' }}>
        <button onClick={onNext} style={{ background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, padding: '10px 32px', fontSize: 13, fontWeight: 700, cursor: 'pointer', letterSpacing: 1 }}>
          NEW USER
        </button>
        <button onClick={onJoin} style={{ background: 'transparent', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 4, padding: '10px 32px', fontSize: 13, fontWeight: 700, cursor: 'pointer', letterSpacing: 1 }}>
          JOIN EXISTING
        </button>
      </div>
    </div>
  )
}
