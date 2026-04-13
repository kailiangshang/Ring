import type { GraphPreview } from '../../types'
import './GraphPreviewCard.css'

interface GraphPreviewCardProps {
  graph: GraphPreview
  on_edit?: () => void
}

export function GraphPreviewCard({ graph, on_edit }: GraphPreviewCardProps) {
  const node_types = [...new Set(graph.nodes.map((n) => n.node_type))]
  const relation_types = [...new Set(graph.edges.map((e) => e.relation))]

  return (
    <div className="graph-preview-card">
      <div className="graph-preview-card-header">
        <div className="graph-preview-card-name">{graph.name}</div>
        {on_edit && (
          <button className="graph-preview-card-edit" onClick={on_edit}>✏️ 编辑</button>
        )}
      </div>
      <div className="graph-preview-card-stats">
        <span className="graph-preview-stat">{graph.nodes.length} 节点</span>
        <span className="graph-preview-stat">{graph.edges.length} 边</span>
      </div>
      {graph.nodes.length > 0 && (
        <div className="graph-preview-card-nodes">
          {graph.nodes.slice(0, 8).map((node) => (
            <span key={node.id} className={`graph-preview-node graph-preview-node-${node.node_type}`}>
              {node.label}
            </span>
          ))}
          {graph.nodes.length > 8 && (
            <span className="graph-preview-more">+{graph.nodes.length - 8}</span>
          )}
        </div>
      )}
      {node_types.length > 0 && (
        <div className="graph-preview-card-meta">
          <span className="graph-preview-label">类型</span>
          {node_types.map((t) => (
            <span key={t} className="graph-preview-tag">{t}</span>
          ))}
        </div>
      )}
      {relation_types.length > 0 && (
        <div className="graph-preview-card-meta">
          <span className="graph-preview-label">关系</span>
          {relation_types.map((r) => (
            <span key={r} className="graph-preview-tag">{r}</span>
          ))}
        </div>
      )}
    </div>
  )
}
