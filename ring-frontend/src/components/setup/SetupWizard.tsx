import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { StepWelcome } from './StepWelcome'
import { StepIdentity } from './StepIdentity'
import { StepLLM } from './StepLLM'
import { StepGitLab } from './StepGitLab'
import { StepDone } from './StepDone'

export function SetupWizard() {
  const [step, setStep] = useState(0)
  const setSetup = useAppStore((s) => s.setSetup)

  const goNext = () => setStep((s) => Math.min(s + 1, 4))
  const goBack = () => setStep((s) => Math.max(s - 1, 0))

  const handleFinish = () => {
    setSetup(true)
  }

  const steps = [
    <StepWelcome onNext={goNext} />,
    <StepIdentity onNext={goNext} onBack={goBack} />,
    <StepLLM onNext={goNext} onBack={goBack} />,
    <StepGitLab onNext={() => { goNext(); handleFinish() }} onBack={goBack} />,
    <StepDone />,
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
