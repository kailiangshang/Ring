import { useState } from 'react'
import { Button } from '../../components/ui/Button'
import { Input } from '../../components/ui/Input'
import { Modal } from '../../components/ui/Modal'
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

  return (
    <>
      <Button onClick={() => set_open(true)}>Create Ring</Button>
      <Modal
        open={open}
        on_close={() => set_open(false)}
        title="Create Ring"
        footer={
          <>
            <Button variant="secondary" onClick={() => set_open(false)}>Cancel</Button>
            <Button form="create-ring-form" type="submit" disabled={loading || !name.trim()}>Create</Button>
          </>
        }
      >
        <form id="create-ring-form" onSubmit={handle_submit}>
          {error && <p className="setup-error" role="alert">{error}</p>}
          <div className="setup-field">
            <label>Name</label>
            <Input
              type="text"
              value={name}
              onChange={(e) => set_name(e.target.value)}
              placeholder="Ring Group name"
            />
          </div>
          <div className="setup-field">
            <label>Description</label>
            <Input
              type="text"
              value={description}
              onChange={(e) => set_description(e.target.value)}
              placeholder="Optional description"
            />
          </div>
          <div className="setup-field">
            <label>Role Description</label>
            <Input
              input_type="textarea"
              value={role_description}
              onChange={(e) => set_role_description(e.target.value)}
              placeholder="Describe the AI role for this ring"
            />
          </div>
        </form>
      </Modal>
    </>
  )
}
