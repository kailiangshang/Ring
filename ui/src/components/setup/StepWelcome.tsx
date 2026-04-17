interface StepProps {
  onNext: () => void
}

export function StepWelcome({ onNext }: StepProps) {
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
      <button
        onClick={onNext}
        style={{
          background: 'var(--accent-cyan)',
          color: 'var(--bg-base)',
          border: 'none',
          borderRadius: 4,
          padding: '10px 32px',
          fontSize: 13,
          fontWeight: 700,
          cursor: 'pointer',
        }}
      >
        Start Setup
      </button>
    </div>
  )
}
