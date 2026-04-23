import { useEffect, useState } from 'react'
import { api, testLLMConfig } from '../../services/api'

const inputStyle: React.CSSProperties = {
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

const sectionTitle: React.CSSProperties = {
  fontSize: 11,
  fontWeight: 700,
  color: 'var(--text-secondary)',
  marginBottom: 8,
  marginTop: 16,
  paddingBottom: 4,
  borderBottom: '1px solid var(--border)',
}

const smallBtn: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 3,
  padding: '4px 10px',
  fontSize: 10,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

function ResultMsg({ result }: { result: { ok: boolean; message: string } | null }) {
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

export function SuperSettingsPanel() {
  const [llmProvider, setLlmProvider] = useState<string>('openai')
  const [llmModel, setLlmModel] = useState('gpt-4o')
  const [llmApiKey, setLlmApiKey] = useState('')
  const [llmBaseUrl, setLlmBaseUrl] = useState('')
  const [llmTesting, setLlmTesting] = useState(false)
  const [llmResult, setLlmResult] = useState<{ ok: boolean; message: string } | null>(null)
  const [llmSaving, setLlmSaving] = useState(false)

  const [gitlabUrl, setGitlabUrl] = useState('')
  const [gitlabToken, setGitlabToken] = useState('')
  const [gitlabTesting, setGitlabTesting] = useState(false)
  const [gitlabResult, setGitlabResult] = useState<{ ok: boolean; message: string } | null>(null)

  useEffect(() => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => {
        setLlmProvider(res.provider)
        setLlmModel(res.model)
        setLlmBaseUrl(res.base_url || '')
      })
      .catch(() => {})
  }, [])

  const handleLlmTest = async () => {
    setLlmTesting(true)
    setLlmResult(null)
    try {
      const result = await testLLMConfig({
        provider: llmProvider,
        model: llmModel,
        api_key: llmApiKey || undefined,
        base_url: llmBaseUrl || undefined,
      })
      setLlmResult(result)
    } catch (e: unknown) {
      setLlmResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setLlmTesting(false)
    }
  }

  const handleLlmSave = async () => {
    setLlmSaving(true)
    try {
      const body: Record<string, string> = { provider: llmProvider, model: llmModel }
      if (llmApiKey) body.api_key = llmApiKey
      if (llmBaseUrl) body.base_url = llmBaseUrl
      await api.put('/config/llm', body)
      setLlmResult({ ok: true, message: 'LLM config saved' })
    } catch (e: unknown) {
      setLlmResult({ ok: false, message: e instanceof Error ? e.message : 'Save failed' })
    } finally {
      setLlmSaving(false)
    }
  }

  const handleGitlabTest = async () => {
    if (!gitlabUrl.trim() || !gitlabToken.trim()) {
      setGitlabResult({ ok: false, message: 'URL and Token required' })
      return
    }
    setGitlabTesting(true)
    setGitlabResult(null)
    try {
      const res = await fetch('/api/config/gitlab/test', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: gitlabUrl, token: gitlabToken }),
      })
      const data = await res.json()
      setGitlabResult(data)
    } catch (e: unknown) {
      setGitlabResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setGitlabTesting(false)
    }
  }

  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ ...sectionTitle, marginTop: 0 }}>LLM Config</p>
      <div style={{ display: 'flex', gap: 4, marginBottom: 8 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => setLlmProvider(p)}
            style={{
              background: llmProvider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: llmProvider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '4px 10px',
              fontSize: 10,
              cursor: 'pointer',
              fontWeight: llmProvider === p ? 700 : 400,
            }}
          >
            {p}
          </button>
        ))}
      </div>
      <label htmlFor="ss-model" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Model</label>
      <input id="ss-model" value={llmModel} onChange={(e) => setLlmModel(e.target.value)} style={inputStyle} />
      <label htmlFor="ss-apikey" style={{ fontSize: 10, color: 'var(--text-dim)' }}>API Key</label>
      <input id="ss-apikey" type="password" value={llmApiKey} onChange={(e) => setLlmApiKey(e.target.value)} placeholder="Leave blank to keep current" style={inputStyle} />
      <label htmlFor="ss-baseurl" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Base URL</label>
      <input id="ss-baseurl" value={llmBaseUrl} onChange={(e) => setLlmBaseUrl(e.target.value)} style={inputStyle} />
      <div style={{ display: 'flex', gap: 6, marginBottom: 8 }}>
        <button onClick={handleLlmTest} disabled={llmTesting} style={{ ...smallBtn, border: '1px solid var(--accent-cyan)', color: 'var(--accent-cyan)', background: 'transparent', opacity: llmTesting ? 0.5 : 1 }}>
          {llmTesting ? 'TESTING...' : 'TEST'}
        </button>
        <button onClick={handleLlmSave} disabled={llmSaving} style={{ ...smallBtn, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderColor: 'var(--accent-cyan)', opacity: llmSaving ? 0.5 : 1 }}>
          SAVE
        </button>
      </div>
      <ResultMsg result={llmResult} />

      <p style={sectionTitle}>GitLab Config <span style={{ color: 'var(--text-dim)', fontWeight: 400 }}>(Optional)</span></p>
      <label htmlFor="ss-gitlab-url" style={{ fontSize: 10, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input id="ss-gitlab-url" value={gitlabUrl} onChange={(e) => setGitlabUrl(e.target.value)} placeholder="https://gitlab.example.com" style={inputStyle} />
      <label htmlFor="ss-gitlab-token" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Personal Access Token</label>
      <input id="ss-gitlab-token" type="password" value={gitlabToken} onChange={(e) => setGitlabToken(e.target.value)} placeholder="glpat-xxx" style={inputStyle} />
      <button onClick={handleGitlabTest} disabled={gitlabTesting} style={{ ...smallBtn, border: '1px solid var(--accent-cyan)', color: 'var(--accent-cyan)', background: 'transparent', marginBottom: 8, opacity: gitlabTesting ? 0.5 : 1 }}>
        {gitlabTesting ? 'TESTING...' : 'TEST CONNECTION'}
      </button>
      <ResultMsg result={gitlabResult} />
    </div>
  )
}
