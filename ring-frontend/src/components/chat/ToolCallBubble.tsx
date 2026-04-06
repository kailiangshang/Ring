export function ToolCallBubble({ tool_name, input, done }: {
  tool_name: string
  input?: unknown
  done: boolean
}) {
  return (
    <div style={{
      padding: '8px 12px',
      borderRadius: 8,
      backgroundColor: '#e8f4f8',
      borderLeft: '3px solid #2196f3',
      marginBottom: 4,
      fontSize: 13,
    }}>
      <span>{done ? '✓' : '⟳'} {tool_name}</span>
      {input != null && (
        <details style={{ marginTop: 4, cursor: 'pointer' }}>
          <summary>Input</summary>
          <pre style={{ fontSize: 11, margin: 0 }}>{String(JSON.stringify(input, null, 2))}</pre>
        </details>
      )}
    </div>
  )
}
