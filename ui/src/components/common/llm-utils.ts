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
