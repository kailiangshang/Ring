import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { TokenUsageBar } from './TokenUsageBar'

describe('TokenUsageBar', () => {
  it('renders token count and limit', () => {
    render(<TokenUsageBar token_count={5000} token_limit={100000} />)
    expect(screen.getByText('5.0k / 100.0k')).toBeInTheDocument()
  })

  it('renders small numbers without k suffix', () => {
    render(<TokenUsageBar token_count={500} token_limit={100000} />)
    expect(screen.getByText('500 / 100.0k')).toBeInTheDocument()
  })

  it('shows warning icon when usage >= 80%', () => {
    render(<TokenUsageBar token_count={85000} token_limit={100000} />)
    expect(screen.getByText('⚠')).toBeInTheDocument()
  })

  it('does not show warning when usage < 80%', () => {
    render(<TokenUsageBar token_count={5000} token_limit={100000} />)
    expect(screen.queryByText('⚠')).not.toBeInTheDocument()
  })

  it('clamps percentage to 100', () => {
    const { container } = render(<TokenUsageBar token_count={200000} token_limit={100000} />)
    const fill = container.querySelector('.token-usage-fill')
    expect(fill).toBeTruthy()
    expect(fill!.getAttribute('style')).toContain('100%')
  })
})
