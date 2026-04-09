import { useEffect } from 'react'
import { useSetupStore } from '../../stores/setupStore'
import { StepUsername } from './StepUsername'
import { StepLlm } from './StepLlm'
import { StepGitlab } from './StepGitlab'
import { Button } from '../../components/ui/Button'
import { useNavigate } from 'react-router-dom'
import './Setup.css'

const STEPS = ['Username', 'LLM', 'GitLab']

export function SetupWizard() {
  const step = useSetupStore((s) => s.step)
  const set_step = useSetupStore((s) => s.set_step)
  const error = useSetupStore((s) => s.error)
  const complete = useSetupStore((s) => s.complete)
  const redirect_home = useSetupStore((s) => s.redirect_home)
  const navigate = useNavigate()

  useEffect(() => {
    if (redirect_home) navigate('/', { replace: true })
  }, [redirect_home, navigate])

  const handle_complete = async () => {
    await complete()
    navigate('/')
  }

  const render_step = () => {
    switch (step) {
      case 0:
        return <StepUsername />
      case 1:
        return (
          <>
            <StepLlm />
            <div className="setup-actions">
              <Button variant="secondary" onClick={() => set_step(0)}>Back</Button>
            </div>
          </>
        )
      case 2:
        return (
          <>
            <StepGitlab />
            <div className="setup-actions">
              <Button variant="secondary" onClick={() => set_step(1)}>Back</Button>
              <Button variant="primary" onClick={handle_complete}>Complete Setup</Button>
            </div>
          </>
        )
      default:
        return null
    }
  }

  return (
    <div className="setup-wrapper">
      <div className="setup-card">
        <h1 className="setup-title">Welcome to Ring</h1>

        <div className="setup-steps">
          {STEPS.map((label, i) => (
            <span key={label} className="setup-step-row">
              {i > 0 && <span className={`setup-step-line${i <= step ? ' completed' : ''}`} />}
              <span className={`setup-step-dot${i === step ? ' active' : ''}`} />
            </span>
          ))}
        </div>
        <div className="setup-step-labels">
          {STEPS.map((label, i) => (
            <span key={label} className={`setup-step-label${i === step ? ' active' : ''}`}>
              {label}
            </span>
          ))}
        </div>

        {render_step()}

        {error && <p className="setup-error" role="alert">{error}</p>}
      </div>
    </div>
  )
}
