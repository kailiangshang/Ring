import { useState } from 'react'
import { ChatView } from '../../pages/RingSpace/ChatView'
import { NodeTree } from '../graph/NodeTree'
import { useGraphStore } from '../../stores/graphStore'
import { Badge } from '../ui/Badge'
import type { GraphNode } from '../../types'
import './CenterView.css'

export function CenterView() {
  const { nodes } = useGraphStore()
  const [show_tree, set_show_tree] = useState(true)
  const [selected_node_id, set_selected_node_id] = useState<string | null>(null)
  const [detail_node, set_detail_node] = useState<GraphNode | null>(null)

  const handle_node_select = (node_id: string) => {
    set_selected_node_id(node_id)
    const node = nodes.find((n) => n.id === node_id)
    if (node && detail_node?.id === node_id) {
      set_detail_node(null)
    } else {
      set_detail_node(node || null)
    }
  }

  return (
    <div className="center-view">
      <div className={`center-tree${show_tree ? '' : ' center-tree-collapsed'}`}>
        <button className="center-tree-toggle" onClick={() => set_show_tree(!show_tree)}>
          {show_tree ? '◀ 收起' : '▶ 节点'}
        </button>
        {show_tree && (
          <div className="center-tree-content">
            <NodeTree nodes={nodes} selected_node_id={selected_node_id} on_select={handle_node_select} />
          </div>
        )}
        {show_tree && detail_node && (
          <div className="center-tree-detail">
            <div className="center-tree-detail-header">
              <span className="center-tree-detail-title">{detail_node.label}</span>
              <button className="center-tree-detail-close" onClick={() => set_detail_node(null)}>✕</button>
            </div>
            <Badge variant="neutral">{detail_node.node_type}</Badge>
            {detail_node.description && <div className="center-tree-detail-desc">{detail_node.description}</div>}
          </div>
        )}
      </div>
      <div className="center-chat">
        <ChatView />
      </div>
    </div>
  )
}
