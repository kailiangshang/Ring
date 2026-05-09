import type { SetupData } from './SetupWizard'
import { LLMConfigForm, ResultMsg } from '../common/LLMConfigForm'
import { useLLMTest, defaultModel, type LLMFormState } from '../common/llm-utils'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
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

export function StepLLM({ data, onChange, onNext, onBack }: StepProps) {
  const form: LLMFormState = {
    provider: data.llm_provider,
    model: data.llm_model,
    api_key: data.llm_api_key,
    base_url: data.llm_base_url,
  }
  const { testing, result: testResult, test } = useLLMTest()

  const handleProviderChange = (p: Partial<LLMFormState>) => {
    if (p.provider) {
      onChange({ llm_provider: p.provider, llm_model: defaultModel(p.provider) })
    } else {
      const mapping: Record<string, string> = { model: 'llm_model', api_key: 'llm_api_key', base_url: 'llm_base_url' }
      const updates: Record<string, string> = {}
      for (const [k, v] of Object.entries(p)) {
        if (mapping[k]) updates[mapping[k]] = v as string
      }
      onChange(updates)
    }
  }

  const provider = data.llm_provider

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM Config
      </h2>

      <LLMConfigForm value={form} onChange={handleProviderChange} labelFontSize={11} idPrefix="setup" hideApiKeyForOllama />

      <button
        onClick={() => test(form)}
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

      <ResultMsg result={testResult} />

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
