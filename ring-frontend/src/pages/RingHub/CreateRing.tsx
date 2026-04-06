import { useState } from 'react'
import type { CreateRingRequest } from '../../types'

interface CreateRingProps {
  on_create: (req: CreateRingRequest) => Promise<void>
}

export function CreateRing({ on_create }: CreateRingProps) {
  const [open, set_open] = useState(false)
  const [name, set_name] = useState('')
  const [description, set_description] = useState('')
  const [role_description, set_role_description] = useState('')
  const [loading, set_loading] = useState(false)
  const [error, set_error] = useState<string | null>(null)

  const handle_submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) return
    set_loading(true)
    set_error(null)
    try {
      await on_create({
        name: name.trim(),
        description: description.trim() || undefined,
        role_description: role_description.trim() || undefined,
      })
      set_name('')
      set_description('')
      set_role_description('')
      set_open(false)
    } catch (err) {
      set_error((err as Error).message)
    } finally {
      set_loading(false)
    }
  }

  if (!open) {
    return <button onClick={() => set_open(true)}>Create Ring</button>
  }

  return (
    <form onSubmit={handle_submit}>
      <h2>Create New Ring</h2>
      <div>
        <label>Name</label>
        <input
          type="text"
          value={name}
          onChange={(e) => set_name(e.target.value)}
          placeholder="Ring name"
        />
      </div>
      <div>
        <label>Description</label>
        <input
          type="text"
          value={description}
          onChange={(e) => set_description(e.target.value)}
          placeholder="Optional description"
        />
      </div>
      <div>
        <label>Role Description</label>
        <textarea
          value={role_description}
          onChange={(e) => set_role_description(e.target.value)}
          placeholder="Describe the AI role for this ring"
        />
      </div>
      {error && <p role="alert">{error}</p>}
      <button type="submit" disabled={loading}>
        Create
      </button>
      <button type="button" onClick={() => set_open(false)}>
        Cancel
      </button>
    </form>
  )
}
