import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CreateRing } from './CreateRing'

describe('CreateRing', () => {
  it('shows button initially, opens form on click', async () => {
    const user = userEvent.setup()
    render(<CreateRing on_create={vi.fn()} />)
    expect(screen.getByText('Create Ring')).toBeInTheDocument()
    await user.click(screen.getByText('Create Ring'))
    expect(screen.getByText('Create New Ring')).toBeInTheDocument()
  })

  it('calls on_create with form data', async () => {
    const on_create = vi.fn().mockResolvedValue(undefined)
    const user = userEvent.setup()
    render(<CreateRing on_create={on_create} />)
    await user.click(screen.getByText('Create Ring'))
    await user.type(screen.getByPlaceholderText('Ring name'), 'Test Ring')
    await user.click(screen.getByRole('button', { name: 'Create' }))
    expect(on_create).toHaveBeenCalledWith({
      name: 'Test Ring',
      description: undefined,
      role_description: undefined,
    })
  })

  it('shows cancel button that closes form', async () => {
    const user = userEvent.setup()
    render(<CreateRing on_create={vi.fn()} />)
    await user.click(screen.getByText('Create Ring'))
    expect(screen.getByText('Create New Ring')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Cancel' }))
    expect(screen.queryByText('Create New Ring')).not.toBeInTheDocument()
  })
})
