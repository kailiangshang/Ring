import { useState } from 'react'
import type { LLMProvider } from '../../types/config'
import type { SetupData } from './SetupWizard'
import { testLLMConfig } from '../../services/api'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
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

const defaultModel = (p: string) => {
  if (p === 'anthropic') return 'claude-sonnet-4-20250514'
  if (p === 'ollama') return 'qwen2.5'
  return 'gpt-4o'
}

export function StepLLM({ data, onChange, onNext, onBack }: StepProps) {
  const provider = data.llm_provider as LLMProvider
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null)

  const handleProviderChange = (p: string) => {
    onChange({ llm_provider: p, llm_model: defaultModel(p) })
    setTestResult(null)
  }

  const handleTest = async () => {
    setTesting(true)
    setTestResult(null)
    try {
      const result = await testLLMConfig({
        provider: data.llm_provider,
        model: data.llm_model,
        api_key: provider !== 'ollama' ? data.llm_api_key || undefined : undefined,
        base_url: data.llm_base_url || undefined,
      })
      setTestResult(result)
    } catch (e: unknown) {
      setTestResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setTesting(false)
    }
  }

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM Config
      </h2>

      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => handleProviderChange(p)}
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

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>Model</label>
      <input
        value={data.llm_model}
        onChange={(e) => { onChange({ llm_model: e.target.value }); setTestResult(null) }}
        placeholder={defaultModel(provider)}
        style={inputStyle}
      />

      {provider !== 'ollama' && (
        <>
          <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>API Key</label>
          <input
            type="password"
            value={data.llm_api_key}
            onChange={(e) => { onChange({ llm_api_key: e.target.value }); setTestResult(null) }}
            placeholder={`sk-${provider === 'openai' ? 'xxx' : 'ant-xxx'}`}
            style={inputStyle}
          />
        </>
      )}

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Base URL {provider === 'ollama' ? '(e.g. http://localhost:11434)' : '(optional)'}
      </label>
      <input
        value={data.llm_base_url}
        onChange={(e) => { onChange({ llm_base_url: e.target.value }); setTestResult(null) }}
        placeholder={provider === 'ollama' ? 'http://localhost:11434' : ''}
        style={inputStyle}
      />

      <button
        onClick={handleTest}
        disabled={testing}
        style={{
          ...navButtonStyle,
          width: '100%',
          marginBottom: 8,
          opacity: testing ? 0.5 : 1,
          border: '1px solid var(--accent-cyan)',
          color: 'var(--accent-cyan)',
          background: 'transparent',
        }}
      >
        {testing ? 'TESTING...' : 'TEST CONNECTION'}
      </button>

      {testResult && (
        <div style={{
          fontSize: 11,
          padding: '6px 8px',
          borderRadius: 3,
          marginBottom: 12,
          background: testResult.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
          color: testResult.ok ? 'var(--accent-green)' : '#ef4444',
          border: `1px solid ${testResult.ok ? 'var(--accent-green)' : '#ef4444'}`,
        }}>
          {testResult.message}
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={provider !== 'ollama' && !data.llm_api_key.trim()}
          style={{
            ...navButtonStyle,
            opacity: provider !== 'ollama' && !data.llm_api_key.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
