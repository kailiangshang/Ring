import { useState, useCallback } from 'react'
import { testLLMConfig } from '../../services/api'

export const defaultModel = (p: string) => {
  if (p === 'anthropic') return 'claude-sonnet-4-20250514'
  if (p === 'ollama') return 'qwen2.5'
  return 'gpt-4o'
}

export const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '6px 10px',
  color: 'var(--text-primary)',
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 8,
  marginTop: 2,
}

export interface LLMFormState {
  provider: string
  model: string
  api_key: string
  base_url: string
}

interface LLMConfigFormProps {
  value: LLMFormState
  onChange: (partial: Partial<LLMFormState>) => void
  labelFontSize?: number
  idPrefix?: string
  hideApiKeyForOllama?: boolean
}

export function LLMConfigForm({ value, onChange, labelFontSize = 10, idPrefix = 'llm', hideApiKeyForOllama = false }: LLMConfigFormProps) {
  return (
    <>
      <div style={{ display: 'flex', gap: 4, marginBottom: 8 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => onChange({ provider: p })}
            style={{
              background: value.provider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: value.provider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '4px 10px',
              fontSize: 10,
              cursor: 'pointer',
              fontWeight: value.provider === p ? 700 : 400,
            }}
          >
            {p}
          </button>
        ))}
      </div>

      <label htmlFor={`${idPrefix}-model`} style={{ fontSize: labelFontSize, color: 'var(--text-dim)' }}>Model</label>
      <input id={`${idPrefix}-model`} value={value.model} onChange={(e) => onChange({ model: e.target.value })} style={inputStyle} />

      {(!hideApiKeyForOllama || value.provider !== 'ollama') && (
        <>
          <label htmlFor={`${idPrefix}-apikey`} style={{ fontSize: labelFontSize, color: 'var(--text-dim)' }}>API Key</label>
          <input
            id={`${idPrefix}-apikey`}
            type="password"
            value={value.api_key}
            onChange={(e) => onChange({ api_key: e.target.value })}
            placeholder="Leave blank to keep current"
            style={inputStyle}
          />
        </>
      )}

      <label htmlFor={`${idPrefix}-baseurl`} style={{ fontSize: labelFontSize, color: 'var(--text-dim)' }}>Base URL</label>
      <input id={`${idPrefix}-baseurl`} value={value.base_url} onChange={(e) => onChange({ base_url: e.target.value })} style={inputStyle} />
    </>
  )
}

export function useLLMTest() {
  const [testing, setTesting] = useState(false)
  const [result, setResult] = useState<{ ok: boolean; message: string } | null>(null)

  const test = useCallback(async (form: LLMFormState) => {
    setTesting(true)
    setResult(null)
    try {
      const r = await testLLMConfig({
        provider: form.provider,
        model: form.model,
        api_key: form.api_key || undefined,
        base_url: form.base_url || undefined,
      })
      setResult(r)
    } catch (e: unknown) {
      setResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setTesting(false)
    }
  }, [])

  return { testing, result, setResult, test }
}

export function ResultMsg({ result }: { result: { ok: boolean; message: string } | null }) {
  if (!result) return null
  return (
    <div style={{
      fontSize: 10,
      padding: '4px 8px',
      borderRadius: 3,
      marginBottom: 8,
      background: result.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
      color: result.ok ? 'var(--accent-green)' : '#ef4444',
      border: `1px solid ${result.ok ? 'var(--accent-green)' : '#ef4444'}`,
    }}>
      {result.message}
    </div>
  )
}
