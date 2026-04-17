import { useState } from 'react'
import type { LLMProvider } from '../../types/config'

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

export function StepLLM({ onNext, onBack }: StepProps) {
  const [provider, setProvider] = useState<LLMProvider>('openai')
  const [apiKey, setApiKey] = useState('')
  const [baseUrl, setBaseUrl] = useState('')

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM Config
      </h2>

      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => setProvider(p)}
            style={{
              background: provider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: provider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 14px',
              fontSize: 12,
              cursor: 'pointer',
              fontWeight: provider === p ? 700 : 400,
            }}
          >
            {p}
          </button>
        ))}
      </div>

      {provider !== 'ollama' && (
        <>
          <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>API Key</label>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={`sk-${provider === 'openai' ? 'xxx' : 'ant-xxx'}`}
            style={inputStyle}
          />
        </>
      )}

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Base URL {provider === 'ollama' ? '(e.g. http://localhost:11434)' : '(optional)'}
      </label>
      <input
        value={baseUrl}
        onChange={(e) => setBaseUrl(e.target.value)}
        placeholder={provider === 'ollama' ? 'http://localhost:11434' : ''}
        style={inputStyle}
      />

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={provider !== 'ollama' && !apiKey.trim()}
          style={{
            ...navButtonStyle,
            opacity: provider !== 'ollama' && !apiKey.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
