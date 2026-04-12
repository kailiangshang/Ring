import { vi } from 'vitest'
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { TabBar } from './TabBar'

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useParams: () => ({ ringId: 'ring-1' }),
    useLocation: () => ({ pathname: '/ring/ring-1' }),
  }
})

describe('TabBar', () => {
  it('renders all 5 tabs', () => {
    render(<TabBar />)
    expect(screen.getByText('Chat')).toBeInTheDocument()
    expect(screen.getByText('Graph')).toBeInTheDocument()
    expect(screen.getByText('PRs')).toBeInTheDocument()
    expect(screen.getByText('Members')).toBeInTheDocument()
    expect(screen.getByText('Sessions')).toBeInTheDocument()
  })
})
