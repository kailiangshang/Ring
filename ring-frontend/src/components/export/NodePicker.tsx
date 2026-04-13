import type { GraphNode } from '../../types'
import './NodePicker.css'

interface NodePickerProps {
  nodes: GraphNode[]
  selected: string | string[]
  on_select: (value: string | string[]) => void
  multiple: boolean
}

export function NodePicker({ nodes, selected, on_select, multiple }: NodePickerProps) {
  const is_selected = (id: string) =>
    Array.isArray(selected) ? selected.includes(id) : selected === id

  const handle_click = (id: string) => {
    if (multiple) {
      const current = Array.isArray(selected) ? selected : []
      const next = current.includes(id)
        ? current.filter((x) => x !== id)
        : [...current, id]
      on_select(next)
    } else {
      on_select(id)
    }
  }

  return (
    <div className="node-picker">
      {nodes.map((node) => (
        <button
          key={node.id}
          className={`node-picker-item${is_selected(node.id) ? ' node-picker-selected' : ''}`}
          onClick={() => handle_click(node.id)}
        >
          <span className="node-picker-type">{node.node_type}</span>
          <span className="node-picker-label">{node.label}</span>
        </button>
      ))}
    </div>
  )
}
