export function ToolResultBubble({ tool_name, output, success }: {
  tool_name: string
  output?: unknown
  success?: boolean
}) {
  return (
    <div style={{
      padding: '8px 12px',
      borderRadius: 8,
      backgroundColor: success ? '#e8f5e9' : '#fbe9e7',
      borderLeft: `3px solid ${success ? '#4caf50' : '#f44336'}`,
      marginBottom: 4,
      fontSize: 13,
    }}>
      <span>{tool_name} → {success ? 'Success' : 'Error'}</span>
      {output != null && (
        <details style={{ marginTop: 4, cursor: 'pointer' }}>
          <summary>Output</summary>
          <pre style={{ fontSize: 11, margin: 0, maxHeight: 200, overflow: 'auto' }}>
            {typeof output === 'string' ? output : String(JSON.stringify(output, null, 2))}
          </pre>
        </details>
      )}
    </div>
  )
}
