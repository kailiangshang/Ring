import { useState } from 'react'
import { useSetupStore } from '../../stores/setupStore'

export function StepUsername() {
  const [display_name, set_display_name] = useState('')
  const [local_error, set_local_error] = useState('')
  const submit_username = useSetupStore((s) => s.submit_username)
  const loading = useSetupStore((s) => s.loading)

  const handle_submit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!display_name.trim()) {
      set_local_error('Display name is required')
      return
    }
    set_local_error('')
    submit_username(display_name.trim())
  }

  return (
    <form onSubmit={handle_submit}>
      <h2>Set Your Display Name</h2>
      <input
        type="text"
        value={display_name}
        onChange={(e) => set_display_name(e.target.value)}
        placeholder="Your name"
        disabled={loading}
      />
      {local_error && <p role="alert">{local_error}</p>}
      <button type="submit" disabled={loading}>
        Next
      </button>
    </form>
  )
}
