import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArchiveSuggestion } from './ArchiveSuggestion'

describe('ArchiveSuggestion', () => {
  it('renders reason and suggested title', () => {
    render(
      <ArchiveSuggestion
        data={{ reason: '有价值的信息', suggested_title: '会议纪要' }}
        on_accept={vi.fn()}
        on_dismiss={vi.fn()}
      />,
    )
    expect(screen.getByText('有价值的信息')).toBeInTheDocument()
    expect(screen.getByText('📄 会议纪要')).toBeInTheDocument()
  })

  it('renders parent and action preview', () => {
    render(
      <ArchiveSuggestion
        data={{
          reason: 'test',
          suggested_parent: { id: 'n1', label: '产品' },
          action_preview: '将创建新节点',
        }}
        on_accept={vi.fn()}
        on_dismiss={vi.fn()}
      />,
    )
    expect(screen.getByText('📂 产品')).toBeInTheDocument()
    expect(screen.getByText('将创建新节点')).toBeInTheDocument()
  })

  it('calls on_accept with suggestion data when 归档 clicked', () => {
    const on_accept = vi.fn()
    const data = { reason: 'test', suggested_title: '记录', suggested_parent: { id: 'n1', label: '根' } }
    render(<ArchiveSuggestion data={data} on_accept={on_accept} on_dismiss={vi.fn()} />)
    fireEvent.click(screen.getByText('归档'))
    expect(on_accept).toHaveBeenCalledWith(data)
  })

  it('calls on_dismiss when 跳过 clicked', () => {
    const on_dismiss = vi.fn()
    render(<ArchiveSuggestion data={{}} on_accept={vi.fn()} on_dismiss={on_dismiss} />)
    fireEvent.click(screen.getByText('跳过'))
    expect(on_dismiss).toHaveBeenCalled()
  })
})
