import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ArchiveQueueBar } from './ArchiveQueueBar'
import { useGitStore } from '../../stores/gitStore'

vi.useFakeTimers()

beforeEach(() => {
  useGitStore.setState({
    archive_queue: null,
    loading: false,
    error: null,
    prs: [],
    current_pr: null,
    commit_log: [],
  })
})

describe('ArchiveQueueBar', () => {
  it('shows empty state when queue is empty', () => {
    useGitStore.setState({
      archive_queue: { current_review: null, queue: [] },
    })
    render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(screen.getByText('归档队列空闲')).toBeInTheDocument()
  })

  it('shows current review and queue count', () => {
    useGitStore.setState({
      archive_queue: {
        current_review: { pr_id: 1, author: 'Li', title: '添加笔记', position: 1 },
        queue: [
          { pr_id: 2, author: 'Ming', title: '更新文档', position: 2 },
        ],
      },
    })
    render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(screen.getByText(/正在审核: 添加笔记/)).toBeInTheDocument()
    expect(screen.getByText(/排队中: 1 个/)).toBeInTheDocument()
  })

  it('returns null when archive_queue is null', () => {
    useGitStore.setState({ archive_queue: null })
    const { container } = render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(container.innerHTML).toBe('')
  })
})
