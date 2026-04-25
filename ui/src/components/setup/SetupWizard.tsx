import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useAuthStore } from '../../stores/auth-store'
import { api } from '../../services/api'
import { StepWelcome } from './StepWelcome'
import { StepIdentity } from './StepIdentity'
import { StepLLM } from './StepLLM'
import { StepGitLab } from './StepGitLab'
import { StepDone } from './StepDone'
import { StepJoin } from './StepJoin'

export interface SetupData {
  display_name: string
  avatar: string | null
  llm_provider: string
  llm_api_key: string
  llm_model: string
  llm_base_url: string
  gitlab_url: string
  gitlab_token: string
  github_url: string
  github_token: string
}

interface JoinParams {
  token?: string
  creator_ip?: string
}

export function SetupWizard({ join_params }: { join_params?: JoinParams }) {
  const [step, setStep] = useState(0)
  const [mode, setMode] = useState<'setup' | 'join'>(join_params?.token ? 'join' : 'setup')
  const [data, setData] = useState<SetupData>({
    display_name: '',
    avatar: null,
    llm_provider: 'openai',
    llm_api_key: '',
    llm_model: 'gpt-4o',
    llm_base_url: '',
    gitlab_url: '',
    gitlab_token: '',
    github_url: '',
    github_token: '',
  })
  const [token, setToken] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const setSetup = useAppStore((s) => s.setSetup)
  const setAuth = useAuthStore((s) => s.setAuth)

  const goNext = () => setStep((s) => Math.min(s + 1, 4))
  const goBack = () => setStep((s) => Math.max(s - 1, 0))

  const handleSubmit = async () => {
    setError(null)
    try {
      const res = await api.post<{ token_id: string; display_name: string; avatar: string | null }>('/setup', {
        display_name: data.display_name,
        avatar: data.avatar,
        llm_provider: data.llm_provider,
        llm_api_key: data.llm_provider !== 'ollama' ? data.llm_api_key : null,
        llm_model: data.llm_model || undefined,
        llm_base_url: data.llm_base_url || undefined,
        gitlab_url: data.gitlab_url.trim() || null,
        gitlab_token: data.gitlab_token.trim() || null,
        github_url: data.github_url.trim() || null,
        github_token: data.github_token.trim() || null,
      })
      setToken(res.token_id)
      setAuth(res.token_id, res.display_name, res.avatar)
      goNext()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Setup failed'
      setError(msg)
    }
  }

  if (mode === 'join') {
    return (
      <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)' }}>
        <StepJoin initial_token={join_params?.token} initial_creator_ip={join_params?.creator_ip} />
      </div>
    )
  }

  const steps = [
    <StepWelcome key="welcome" onNext={goNext} onJoin={() => setMode('join')} />,
    <StepIdentity key="identity" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepLLM key="llm" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepGitLab key="gitlab" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={handleSubmit} onBack={goBack} error={error} />,
    <StepDone key="done" token={token} onEnter={() => setSetup(true)} />,
  ]

  return (
    <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)' }}>
      {steps[step]}
    </div>
  )
}
