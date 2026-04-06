import type { FileChange } from '../../types'

interface DiffViewProps {
  changes: FileChange[]
}

export function DiffView({ changes }: DiffViewProps) {
  if (changes.length === 0) {
    return <p style={{ color: '#888' }}>No changes</p>
  }

  return (
    <div>
      {changes.map((change, i) => (
        <div key={i} style={{ marginBottom: '1rem' }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              padding: '0.5rem',
              background: '#f5f5f5',
              borderRadius: '4px 4px 0 0',
              fontFamily: 'monospace',
              fontSize: '0.85rem',
            }}
          >
            <span
              style={{
                padding: '2px 6px',
                borderRadius: '3px',
                fontSize: '0.75rem',
                fontWeight: 600,
                background:
                  change.status === 'added'
                    ? '#d4edda'
                    : change.status === 'deleted'
                      ? '#f8d7da'
                      : '#fff3cd',
                color:
                  change.status === 'added'
                    ? '#155724'
                    : change.status === 'deleted'
                      ? '#721c24'
                      : '#856404',
              }}
            >
              {change.status}
            </span>
            <span>{change.file}</span>
            <span style={{ marginLeft: 'auto', color: '#28a745' }}>
              +{change.additions}
            </span>
            <span style={{ color: '#dc3545' }}>-{change.deletions}</span>
          </div>
          <pre
            style={{
              background: '#fafafa',
              border: '1px solid #e0e0e0',
              borderTop: 'none',
              borderRadius: '0 0 4px 4px',
              padding: '0.75rem',
              overflow: 'auto',
              fontSize: '0.8rem',
              lineHeight: 1.5,
              maxHeight: '400px',
            }}
          >
            {change.diff}
          </pre>
        </div>
      ))}
    </div>
  )
}
