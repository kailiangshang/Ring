import { Badge } from '../ui/Badge'
import type { GraphNode } from '../../types'
import './RightPanel.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

interface RightPanelProps { state: RightPanelState; on_close: () => void }

function NodeDetailView({ node }: { node: GraphNode }) {
  return (
    <div>
      <div className="right-panel-node-title">{node.label}</div>
      <div className="right-panel-node-meta">
        <Badge variant="neutral">{node.node_type}</Badge>
        {node.description && <span className="right-panel-node-desc">{node.description}</span>}
      </div>
      <div className="right-panel-node-path">
        {node.markdown_path || '无关联文档'}
      </div>
    </div>
  )
}

export function RightPanel({ state, on_close }: RightPanelProps) {
  const titles: Record<string, string> = { node_detail: '节点详情', diff: 'Changes', node_selector: '选择节点' }
  const node = state.content === 'node_detail' ? (state.data as GraphNode | null) : null

  return (
    <div className="right-panel">
      <div className="right-panel-header">
        <h4>{state.content ? titles[state.content] || '' : ''}</h4>
        <button className="right-panel-close" onClick={on_close}>&times;</button>
      </div>
      <div className="right-panel-body">
        {node && <NodeDetailView node={node} />}
        {state.content === 'diff' && <div className="right-panel-stub">Diff view</div>}
        {state.content === 'node_selector' && <div className="right-panel-stub">节点选择器</div>}
      </div>
    </div>
  )
}
