import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useAuthStore } from '../../stores/auth-store'
import { api } from '../../services/api'
import { StepWelcome } from './StepWelcome'
import { StepIdentity } from './StepIdentity'
import { StepLLM } from './StepLLM'
import { StepGitLab } from './StepGitLab'
import { StepDone } from './StepDone'

export interface SetupData {
  display_name: string
  avatar: string | null
  llm_provider: string
  llm_api_key: string
  llm_model: string
  llm_base_url: string
  gitlab_url: string
  gitlab_token: string
}

export function SetupWizard() {
  const [step, setStep] = useState(0)
  const [data, setData] = useState<SetupData>({
    display_name: '',
    avatar: null,
    llm_provider: 'openai',
    llm_api_key: '',
    llm_model: 'gpt-4o',
    llm_base_url: '',
    gitlab_url: '',
    gitlab_token: '',
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
        gitlab_url: data.gitlab_url,
        gitlab_token: data.gitlab_token,
      })
      setToken(res.token_id)
      setAuth(res.token_id, res.display_name, res.avatar)
      goNext()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Setup failed'
      setError(msg)
    }
  }

  const steps = [
    <StepWelcome onNext={goNext} />,
    <StepIdentity data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepLLM data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepGitLab data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={handleSubmit} onBack={goBack} error={error} />,
    <StepDone token={token} onEnter={() => setSetup(true)} />,
  ]

  return (
    <div
      style={{
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'var(--bg-base)',
      }}
    >
      {steps[step]}
    </div>
  )
}
