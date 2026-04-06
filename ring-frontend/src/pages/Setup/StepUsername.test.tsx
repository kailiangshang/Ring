import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { StepUsername } from './StepUsername'

vi.mock('../../stores/setupStore', () => ({
  useSetupStore: (selector: (s: Record<string, unknown>) => unknown) =>
    selector({
      submit_username: vi.fn().mockResolvedValue(undefined),
      loading: false,
    }),
}))

describe('StepUsername', () => {
  it('renders input and submit button', () => {
    render(<StepUsername />)
    expect(screen.getByPlaceholderText('Your name')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Next' })).toBeInTheDocument()
  })

  it('shows error on empty input', async () => {
    const user = userEvent.setup()
    render(<StepUsername />)
    await user.click(screen.getByRole('button', { name: 'Next' }))
    expect(screen.getByRole('alert')).toHaveTextContent('Display name is required')
  })

  it('calls submit on valid input', async () => {
    const user = userEvent.setup()
    render(<StepUsername />)
    await user.type(screen.getByPlaceholderText('Your name'), 'Alice')
    await user.click(screen.getByRole('button', { name: 'Next' }))
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })
})
