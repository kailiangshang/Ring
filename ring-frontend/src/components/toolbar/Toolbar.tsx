export interface ToolStatus {
  name: string
  description: string
  active: boolean
}

interface ToolbarProps {
  tools: ToolStatus[]
  on_toggle?: (tool_name: string) => void
}

export function Toolbar({ tools, on_toggle }: ToolbarProps) {
  if (tools.length === 0) return null

  return (
    <div style={{
      display: 'flex',
      gap: 8,
      padding: '6px 0',
      flexWrap: 'wrap',
      borderBottom: '1px solid #e0e0e0',
      marginBottom: 8,
    }}>
      {tools.map((tool) => (
        <button
          key={tool.name}
          onClick={() => on_toggle?.(tool.name)}
          style={{
            padding: '4px 10px',
            borderRadius: 12,
            border: `1px solid ${tool.active ? '#2196f3' : '#ccc'}`,
            backgroundColor: tool.active ? '#e3f2fd' : '#fff',
            color: tool.active ? '#1565c0' : '#666',
            cursor: 'pointer',
            fontSize: 12,
          }}
          title={tool.description}
        >
          {tool.name}
        </button>
      ))}
    </div>
  )
}
