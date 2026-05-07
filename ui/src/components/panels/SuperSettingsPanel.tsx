import { useEffect, useState } from 'react'
import { api } from '../../services/api'
import { useAuthStore } from '../../stores/auth-store'
import { LLMConfigForm, useLLMTest, ResultMsg, inputStyle, type LLMFormState } from '../common/LLMConfigForm'

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

export function SuperSettingsPanel() {
  const [llmForm, setLlmForm] = useState<LLMFormState>({ provider: 'openai', model: 'gpt-4o', api_key: '', base_url: '' })
  const { testing: llmTesting, result: llmResult, test: llmTest } = useLLMTest()
  const [llmSaving, setLlmSaving] = useState(false)

  const [gitlabUrl, setGitlabUrl] = useState('')
  const [gitlabToken, setGitlabToken] = useState('')
  const [gitlabTesting, setGitlabTesting] = useState(false)
  const [gitlabResult, setGitlabResult] = useState<{ ok: boolean; message: string } | null>(null)
  const [rotateResult, setRotateResult] = useState<{ ok: boolean; message: string } | null>(null)
  const setAuth = useAuthStore((s) => s.setAuth)

  useEffect(() => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => {
        setLlmForm((prev) => ({ ...prev, provider: res.provider, model: res.model, base_url: res.base_url || '' }))
      })
      .catch(() => {})
    api.get<{ url: string; token_set: boolean }>('/config/gitlab')
      .then((res) => {
        setGitlabUrl(res.url || '')
      })
      .catch(() => {})
  }, [])

  const handleLlmSave = async () => {
    setLlmSaving(true)
    try {
      const body: Record<string, string> = { provider: llmForm.provider, model: llmForm.model }
      if (llmForm.api_key) body.api_key = llmForm.api_key
      if (llmForm.base_url) body.base_url = llmForm.base_url
      await api.put('/config/llm', body)
    } catch (e: unknown) {
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
      setGitlabResult({
        ok: data.ok === true,
        message: typeof data.message === 'string' ? data.message : 'Unknown response',
      })
    } catch (e: unknown) {
      setGitlabResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setGitlabTesting(false)
    }
  }

  const handleRotateToken = async () => {
    setRotateResult(null)
    try {
      const res = await api.post<{ token_id: string }>('/auth/rotate', {})
      setAuth(res.token_id, '', null)
      setRotateResult({ ok: true, message: 'Token rotated successfully' })
    } catch (e: unknown) {
      setRotateResult({ ok: false, message: e instanceof Error ? e.message : 'Rotation failed' })
    }
  }

  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ ...sectionTitle, marginTop: 0 }}>LLM Config</p>
      <LLMConfigForm value={llmForm} onChange={(p) => setLlmForm((prev) => ({ ...prev, ...p }))} idPrefix="ss" />
      <div style={{ display: 'flex', gap: 6, marginBottom: 8 }}>
        <button onClick={() => llmTest(llmForm)} disabled={llmTesting} style={{ ...smallBtn, border: '1px solid var(--accent-cyan)', color: 'var(--accent-cyan)', background: 'transparent', opacity: llmTesting ? 0.5 : 1 }}>
          {llmTesting ? 'TESTING...' : 'TEST'}
        </button>
        <button onClick={handleLlmSave} disabled={llmSaving} style={{ ...smallBtn, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderColor: 'var(--accent-cyan)', opacity: llmSaving ? 0.5 : 1 }}>
          SAVE
        </button>
      </div>
      <ResultMsg result={llmResult} />

      <p style={sectionTitle}>GitLab Config <span style={{ color: 'var(--text-dim)', fontWeight: 400 }}>(Optional)</span></p>
      <label htmlFor="ss-gitlab-url" style={{ fontSize: 10, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input id="ss-gitlab-url" value={gitlabUrl} onChange={(e) => setGitlabUrl(e.target.value)} placeholder="https://gitlab.company.com" style={inputStyle} />
      <label htmlFor="ss-gitlab-token" style={{ fontSize: 10, color: 'var(--text-dim)' }}>Personal Access Token</label>
      <input id="ss-gitlab-token" type="password" value={gitlabToken} onChange={(e) => setGitlabToken(e.target.value)} placeholder="glpat-xxx" style={inputStyle} />
      <button onClick={handleGitlabTest} disabled={gitlabTesting} style={{ ...smallBtn, border: '1px solid var(--accent-cyan)', color: 'var(--accent-cyan)', background: 'transparent', marginBottom: 8, opacity: gitlabTesting ? 0.5 : 1 }}>
        {gitlabTesting ? 'TESTING...' : 'TEST CONNECTION'}
      </button>
      <ResultMsg result={gitlabResult} />

      <p style={sectionTitle}>Auth Token</p>
      <button onClick={handleRotateToken} style={{ ...smallBtn, border: '1px solid #f59e0b', color: '#f59e0b', background: 'transparent', marginBottom: 8 }}>
        RESET TOKEN
      </button>
      <ResultMsg result={rotateResult} />
    </div>
  )
}
