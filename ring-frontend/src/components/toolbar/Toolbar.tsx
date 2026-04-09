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
    <div className="toolbar">
      {tools.map((tool) => (
        <button
          key={tool.name}
          className={`toolbar-btn${tool.active ? ' toolbar-btn-active' : ''}`}
          onClick={() => on_toggle?.(tool.name)}
          title={tool.description}
        >
          {tool.name}
        </button>
      ))}
    </div>
  )
}
