import type { FileChange } from '../../types'
import './DiffView.css'

interface DiffViewProps {
  changes: FileChange[]
}

export function DiffView({ changes }: DiffViewProps) {
  if (changes.length === 0) {
    return <p className="diff-empty">No changes</p>
  }

  return (
    <div>
      {changes.map((change, i) => (
        <div key={i} className="diff-file">
          <div className="diff-file-header">
            <span className={`diff-status-badge diff-status-${change.status}`}>
              {change.status}
            </span>
            <span className="diff-file-name">{change.file}</span>
            <span className="diff-additions">+{change.additions}</span>
            <span className="diff-deletions">-{change.deletions}</span>
          </div>
          <pre className="diff-content">{change.diff}</pre>
        </div>
      ))}
    </div>
  )
}
