import { useEffect, useState } from 'react'
import { useSettingsStore } from '../../stores/settingsStore'
import { Input } from '../../components/ui/Input'
import { Button } from '../../components/ui/Button'
import './SettingsPage.css'

const LLM_PROVIDERS = [
  { value: 'openai', label: 'OpenAI' },
  { value: 'ollama', label: 'Ollama' },
  { value: 'anthropic', label: 'Anthropic' },
]

export function SettingsPage() {
  const { settings, loading, error, load_settings, save_settings } = useSettingsStore()
  const [form, set_form] = useState({
    llm_provider: '',
    llm_model: '',
    llm_api_key: '',
    llm_base_url: '',
    privacy_enabled: 'false',
  })

  useEffect(() => {
    load_settings()
  }, [])

  useEffect(() => {
    set_form({
      llm_provider: settings.llm_provider || '',
      llm_model: settings.llm_model || '',
      llm_api_key: settings.llm_api_key || '',
      llm_base_url: settings.llm_base_url || '',
      privacy_enabled: settings.privacy_enabled || 'false',
    })
  }, [settings])

  const handle_save = (e: React.FormEvent) => {
    e.preventDefault()
    save_settings(form)
  }

  return (
    <div className="settings-page">
      <h2>Settings</h2>
      {error && <p className="setup-error" role="alert">{error}</p>}

      <div className="settings-card">
        <h3>Profile</h3>
        <div className="settings-row">
          <span className="settings-row-label">Display Name</span>
          <span className="settings-row-value">{settings.display_name || '—'}</span>
        </div>
        <div className="settings-row">
          <span className="settings-row-label">User ID</span>
          <code className="settings-row-value">{settings.user_id || '—'}</code>
        </div>
      </div>

      <form onSubmit={handle_save} className="settings-form">
        <div className="settings-card">
          <h3>LLM Configuration</h3>
          <div className="settings-field">
            <label>Provider</label>
            <Input
              input_type="select"
              value={form.llm_provider}
              onChange={(e) => set_form({ ...form, llm_provider: e.target.value })}
            >
              <option value="">Select provider...</option>
              {LLM_PROVIDERS.map((p) => (
                <option key={p.value} value={p.value}>{p.label}</option>
              ))}
            </Input>
          </div>
          <div className="settings-field">
            <label>Model</label>
            <Input
              value={form.llm_model}
              onChange={(e) => set_form({ ...form, llm_model: e.target.value })}
              placeholder="gpt-4o / llama3 / claude-3-sonnet"
            />
          </div>
          <div className="settings-field">
            <label>API Key</label>
            <Input
              type="password"
              value={form.llm_api_key}
              onChange={(e) => set_form({ ...form, llm_api_key: e.target.value })}
              placeholder="sk-..."
            />
          </div>
          <div className="settings-field">
            <label>Base URL (optional)</label>
            <Input
              value={form.llm_base_url}
              onChange={(e) => set_form({ ...form, llm_base_url: e.target.value })}
              placeholder="http://localhost:11434/v1"
            />
          </div>
          <div className="settings-field settings-checkbox">
            <input
              type="checkbox"
              checked={form.privacy_enabled === 'true'}
              onChange={(e) =>
                set_form({ ...form, privacy_enabled: e.target.checked ? 'true' : 'false' })
              }
            />
            <label>Enable Privacy Filter</label>
          </div>
        </div>
        <Button type="submit" disabled={loading}>
          {loading ? 'Saving...' : 'Save Settings'}
        </Button>
      </form>
    </div>
  )
}
