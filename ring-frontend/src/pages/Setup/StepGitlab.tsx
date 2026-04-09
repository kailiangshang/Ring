import { useState } from 'react'
import { useSetupStore } from '../../stores/setupStore'
import { Input } from '../../components/ui/Input'
import { Button } from '../../components/ui/Button'
import type { GitlabConfig } from '../../types'
import './Setup.css'

export function StepGitlab() {
  const [repo_url, set_repo_url] = useState('')
  const [auth_type, set_auth_type] = useState('ssh_key')
  const [ssh_key_path, set_ssh_key_path] = useState('~/.ssh/id_rsa')
  const submit_gitlab = useSetupStore((s) => s.submit_gitlab)
  const error = useSetupStore((s) => s.error)
  const loading = useSetupStore((s) => s.loading)

  const handle_submit = (e: React.FormEvent) => {
    e.preventDefault()
    const config: GitlabConfig = { repo_url, auth_type, ssh_key_path }
    submit_gitlab(config)
  }

  return (
    <form onSubmit={handle_submit}>
      <h2 className="setup-title setup-step-h2">Configure GitLab</h2>
      <div className="setup-field">
        <label>Repository URL</label>
        <Input
          type="text"
          value={repo_url}
          onChange={(e) => set_repo_url(e.target.value)}
          placeholder="git@gitlab.com:group/repo.git"
        />
      </div>
      <div className="setup-field">
        <label>Auth Type</label>
        <Input
          input_type="select"
          value={auth_type}
          onChange={(e) => set_auth_type(e.target.value)}
        >
          <option value="ssh_key">SSH Key</option>
          <option value="https">HTTPS</option>
        </Input>
      </div>
      <div className="setup-field">
        <label>SSH Key Path</label>
        <Input
          type="text"
          value={ssh_key_path}
          onChange={(e) => set_ssh_key_path(e.target.value)}
        />
      </div>
      {error && <p className="setup-error" role="alert">{error}</p>}
      <div className="setup-actions-end">
        <Button
          type="button"
          variant="secondary"
          disabled={loading}
          onClick={() => submit_gitlab({ repo_url: '', auth_type: 'local', ssh_key_path: undefined })}
        >
          Skip
        </Button>
        <Button type="submit" disabled={loading}>Next</Button>
      </div>
    </form>
  )
}
