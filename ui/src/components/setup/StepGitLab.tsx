import { useState } from 'react'

interface StepProps {
  onNext: () => void
  onBack: () => void
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  fontSize: 13,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 12,
  marginTop: 4,
}

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

export function StepGitLab({ onNext, onBack }: StepProps) {
  const [url, setUrl] = useState('')
  const [token, setToken] = useState('')

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 3: GitLab Config
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="https://gitlab.company.com"
        style={inputStyle}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>Personal Access Token</label>
      <input
        type="password"
        value={token}
        onChange={(e) => setToken(e.target.value)}
        placeholder="glpat-xxx"
        style={inputStyle}
      />

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={!url.trim() || !token.trim()}
          style={{
            ...navButtonStyle,
            opacity: !url.trim() || !token.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Done
        </button>
      </div>
    </div>
  )
}
