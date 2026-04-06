import { useSetupStore } from '../../stores/setupStore'
import { StepUsername } from './StepUsername'
import { StepLlm } from './StepLlm'
import { StepGitlab } from './StepGitlab'
import { useNavigate } from 'react-router-dom'

const STEPS = ['Username', 'LLM', 'GitLab']

export function SetupWizard() {
  const step = useSetupStore((s) => s.step)
  const set_step = useSetupStore((s) => s.set_step)
  const error = useSetupStore((s) => s.error)
  const complete = useSetupStore((s) => s.complete)
  const navigate = useNavigate()

  const handle_complete = async () => {
    await complete()
    navigate('/')
  }

  return (
    <div>
      <h1>Welcome to Ring</h1>
      <div>
        {STEPS.map((label, i) => (
          <span
            key={label}
            style={{ fontWeight: i === step ? 'bold' : 'normal', marginRight: 8 }}
          >
            {i + 1}. {label}
          </span>
        ))}
      </div>
      {step === 0 && <StepUsername />}
      {step === 1 && (
        <>
          <StepLlm />
          <button onClick={() => set_step(0)}>Back</button>
        </>
      )}
      {step === 2 && (
        <>
          <StepGitlab />
          <button onClick={() => set_step(1)}>Back</button>
        </>
      )}
      {error && <p role="alert">{error}</p>}
      {step === 2 && (
        <button onClick={handle_complete}>Complete Setup</button>
      )}
    </div>
  )
}
