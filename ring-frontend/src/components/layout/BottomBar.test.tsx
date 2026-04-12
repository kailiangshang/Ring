import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { BottomBar } from './BottomBar'
import { useModeStore } from '../../stores/modeStore'

describe('BottomBar', () => {
  beforeEach(() => {
    useModeStore.setState({ mode: 'daily' })
  })

  it('renders three mode buttons', () => {
    render(<BottomBar />)
    expect(screen.getByText('日常')).toBeInTheDocument()
    expect(screen.getByText('手动归档')).toBeInTheDocument()
    expect(screen.getByText('Auto')).toBeInTheDocument()
  })

  it('switches mode on click', () => {
    render(<BottomBar />)
    fireEvent.click(screen.getByText('Auto'))
    expect(useModeStore.getState().mode).toBe('auto')
  })

  it('does not show tools when show_tools is false', () => {
    render(<BottomBar tools={[{ name: 'search', description: 'Search', active: true }]} show_tools={false} />)
    expect(screen.queryByText('search')).not.toBeInTheDocument()
  })
})
