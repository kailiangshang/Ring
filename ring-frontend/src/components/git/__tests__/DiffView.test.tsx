import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { DiffView } from '../DiffView'
import type { FileChange } from '../../../types'

describe('DiffView', () => {
  it('renders empty state when no changes', () => {
    render(<DiffView changes={[]} />)
    expect(screen.getByText('No changes')).toBeInTheDocument()
  })

  it('renders file changes with status badges', () => {
    const changes: FileChange[] = [
      {
        file: 'src/main.rs',
        status: 'modified',
        additions: 5,
        deletions: 2,
        diff: '@@ -1,3 +1,6 @@\n+new line\n existing line',
      },
    ]
    render(<DiffView changes={changes} />)
    expect(screen.getByText('modified')).toBeInTheDocument()
    expect(screen.getByText('src/main.rs')).toBeInTheDocument()
    expect(screen.getByText('+5')).toBeInTheDocument()
    expect(screen.getByText('-2')).toBeInTheDocument()
  })

  it('renders multiple file changes', () => {
    const changes: FileChange[] = [
      {
        file: 'a.txt',
        status: 'added',
        additions: 10,
        deletions: 0,
        diff: 'new file content',
      },
      {
        file: 'b.txt',
        status: 'deleted',
        additions: 0,
        deletions: 5,
        diff: 'removed content',
      },
    ]
    render(<DiffView changes={changes} />)
    expect(screen.getByText('added')).toBeInTheDocument()
    expect(screen.getByText('deleted')).toBeInTheDocument()
    expect(screen.getByText('a.txt')).toBeInTheDocument()
    expect(screen.getByText('b.txt')).toBeInTheDocument()
  })
})
