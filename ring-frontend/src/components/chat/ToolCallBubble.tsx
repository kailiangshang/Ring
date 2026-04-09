import './ToolCallBubble.css'

export function ToolCallBubble({ tool_name, input, done }: {
  tool_name: string
  input?: unknown
  done: boolean
}) {
  return (
    <div className="tool-call-bubble">
      <span>{done ? '✓' : '⟳'} {tool_name}</span>
      {input != null && (
        <details className="tool-call-details">
          <summary>Input</summary>
          <pre className="tool-call-pre">{String(JSON.stringify(input, null, 2))}</pre>
        </details>
      )}
    </div>
  )
}
