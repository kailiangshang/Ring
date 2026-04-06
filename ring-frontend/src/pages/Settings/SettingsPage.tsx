import { useEffect, useState } from 'react'
import { useSettingsStore } from '../../stores/settingsStore'

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
    <div style={{ maxWidth: 600, margin: '0 auto', padding: 24 }}>
      <h2>Settings</h2>
      {error && <p style={{ color: 'red' }}>{error}</p>}
      <form onSubmit={handle_save} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        <label>
          LLM Provider
          <input
            value={form.llm_provider}
            onChange={(e) => set_form({ ...form, llm_provider: e.target.value })}
            placeholder="openai / ollama / anthropic"
            style={{ width: '100%', marginTop: 4 }}
          />
        </label>
        <label>
          Model
          <input
            value={form.llm_model}
            onChange={(e) => set_form({ ...form, llm_model: e.target.value })}
            placeholder="gpt-4o / llama3 / claude-3-sonnet"
            style={{ width: '100%', marginTop: 4 }}
          />
        </label>
        <label>
          API Key
          <input
            type="password"
            value={form.llm_api_key}
            onChange={(e) => set_form({ ...form, llm_api_key: e.target.value })}
            placeholder="sk-..."
            style={{ width: '100%', marginTop: 4 }}
          />
        </label>
        <label>
          Base URL (optional)
          <input
            value={form.llm_base_url}
            onChange={(e) => set_form({ ...form, llm_base_url: e.target.value })}
            placeholder="http://localhost:11434/v1"
            style={{ width: '100%', marginTop: 4 }}
          />
        </label>
        <label style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <input
            type="checkbox"
            checked={form.privacy_enabled === 'true'}
            onChange={(e) =>
              set_form({ ...form, privacy_enabled: e.target.checked ? 'true' : 'false' })
            }
          />
          Enable Privacy Filter
        </label>
        <button type="submit" disabled={loading}>
          {loading ? 'Saving...' : 'Save Settings'}
        </button>
      </form>
    </div>
  )
}
