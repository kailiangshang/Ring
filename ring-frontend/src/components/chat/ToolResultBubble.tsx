import './ToolResultBubble.css'

export function ToolResultBubble({ tool_name, output, success }: {
  tool_name: string
  output?: unknown
  success?: boolean
}) {
  return (
    <div className={`tool-result-bubble ${success ? 'tool-result-success' : 'tool-result-error'}`}>
      <span>{tool_name} → {success ? 'Success' : 'Error'}</span>
      {output != null && (
        <details className="tool-result-details">
          <summary>Output</summary>
          <pre className="tool-result-pre">
            {typeof output === 'string' ? output : String(JSON.stringify(output, null, 2))}
          </pre>
        </details>
      )}
    </div>
  )
}
