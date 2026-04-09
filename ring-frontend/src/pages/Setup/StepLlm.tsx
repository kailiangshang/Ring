import { useState } from 'react'
import { useSetupStore } from '../../stores/setupStore'
import { Input } from '../../components/ui/Input'
import { Button } from '../../components/ui/Button'
import type { LlmConfig } from '../../types'
import './Setup.css'

export function StepLlm() {
  const [provider, set_provider] = useState('openai')
  const [model, set_model] = useState('gpt-4')
  const [api_key, set_api_key] = useState('')
  const [base_url, set_base_url] = useState('')
  const submit_llm = useSetupStore((s) => s.submit_llm)
  const error = useSetupStore((s) => s.error)
  const loading = useSetupStore((s) => s.loading)

  const handle_submit = (e: React.FormEvent) => {
    e.preventDefault()
    const config: LlmConfig = {
      provider,
      model,
      api_key,
      base_url: base_url || null,
    }
    submit_llm(config)
  }

  return (
    <form onSubmit={handle_submit}>
      <h2 className="setup-title setup-step-h2">Configure LLM</h2>
      <div className="setup-field">
        <label>Provider</label>
        <Input
          input_type="select"
          value={provider}
          onChange={(e) => set_provider(e.target.value)}
        >
          <option value="openai">OpenAI</option>
          <option value="ollama">Ollama</option>
          <option value="anthropic">Anthropic</option>
        </Input>
      </div>
      <div className="setup-field">
        <label>Model</label>
        <Input
          type="text"
          value={model}
          onChange={(e) => set_model(e.target.value)}
        />
      </div>
      <div className="setup-field">
        <label>API Key</label>
        <Input
          type="password"
          value={api_key}
          onChange={(e) => set_api_key(e.target.value)}
        />
      </div>
      <div className="setup-field">
        <label>Base URL (optional)</label>
        <Input
          type="text"
          value={base_url}
          onChange={(e) => set_base_url(e.target.value)}
        />
      </div>
      {error && <p className="setup-error" role="alert">{error}</p>}
      <div className="setup-actions-end">
        <Button type="submit" disabled={loading}>Next</Button>
      </div>
    </form>
  )
}
