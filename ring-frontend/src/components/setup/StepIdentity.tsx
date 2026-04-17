import { useState } from 'react'

interface StepProps {
  onNext: () => void
  onBack: () => void
}

const EMOJIS = ['🦊', '🐱', '🌟', '🚀', '🎯', '💡', '🔥', '🌈', '⚡', '🍀', '🦋', '🎪']

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function StepIdentity({ onNext, onBack }: StepProps) {
  const [name, setName] = useState('')
  const [avatar, setAvatar] = useState<string | null>(null)

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 1: Identity
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)', letterSpacing: '0.05em' }}>
        Display Name
      </label>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Enter your name"
        style={{
          width: '100%',
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 12px',
          color: 'var(--text-primary)',
          fontSize: 13,
          fontFamily: 'inherit',
          outline: 'none',
          marginBottom: 16,
          marginTop: 4,
        }}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)', letterSpacing: '0.05em' }}>
        Avatar
      </label>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 4 }}>
        {EMOJIS.map((emoji) => (
          <button
            key={emoji}
            onClick={() => setAvatar(emoji)}
            style={{
              width: 36,
              height: 36,
              background: avatar === emoji ? 'var(--accent-amber)' : 'var(--bg-hover)',
              border: avatar === emoji ? '2px solid var(--accent-amber)' : '1px solid var(--border)',
              borderRadius: 4,
              fontSize: 18,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            {emoji}
          </button>
        ))}
      </div>

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>
          Back
        </button>
        <button
          onClick={onNext}
          disabled={!name.trim()}
          style={{
            ...navButtonStyle,
            opacity: name.trim() ? 1 : 0.4,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
