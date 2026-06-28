import { useEffect, useCallback } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { GraphCanvas } from './GraphCanvas'

export function GraphPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const nodes = useGraphStore((s) => s.nodes)
  const edges = useGraphStore((s) => s.edges)
  const loading = useGraphStore((s) => s.loading)
  const selected_node_id = useGraphStore((s) => s.selected_node_id)
  const fetchGraph = useGraphStore((s) => s.fetchGraph)
  const selectNode = useGraphStore((s) => s.selectNode)
  const collapsed_nodes = useGraphStore((s) => s.collapsed_nodes)
  const toggleCollapse = useGraphStore((s) => s.toggleCollapse)
  const toggleFloat = useGraphStore((s) => s.toggleFloat)
  const graphs = useGraphStore((s) => s.graphs)
  const switchGraph = useGraphStore((s) => s.switchGraph)
  const graph_id = useGraphStore((s) => s.graph_id)

  useEffect(() => {
    if (active_ring_id) fetchGraph(active_ring_id)
  }, [active_ring_id, fetchGraph])

  const handleSelectNode = useCallback((nodeId: string | null) => {
    selectNode(nodeId)
  }, [selectNode])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)

  if (loading) {
    return <div style={{ padding: 16, color: 'var(--text-dim)', fontSize: 12 }}>Loading graph...</div>
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '6px 10px', borderBottom: '1px solid var(--border)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
          {graphs.length > 1 && graphs.map((g) => (
            <button
              key={g.id}
              onClick={() => active_ring_id && switchGraph(active_ring_id, g.id)}
              style={{
                fontSize: 9,
                padding: '2px 6px',
                borderRadius: 3,
                border: `1px solid ${g.id === graph_id ? 'var(--accent-cyan)' : 'var(--border)'}`,
                background: g.id === graph_id ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: g.id === graph_id ? 'var(--bg-base)' : 'var(--text-secondary)',
                cursor: 'pointer',
                fontWeight: g.id === graph_id ? 700 : 400,
              }}
            >
              {g.name}
            </button>
          ))}
          <span style={{ fontSize: 9, color: 'var(--text-dim)' }}>{nodes.length} nodes · {edges.length} edges</span>
        </div>
        <button
          onClick={() => toggleFloat()}
          style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 4, padding: '2px 8px', fontSize: 10, cursor: 'pointer' }}
          title="Open fullscreen"
        >
          ⛶
        </button>
      </div>

      <div style={{ flex: 1, minHeight: 0 }}>
        {nodes.length === 0 ? (
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-dim)', fontSize: 12 }}>
            No graph data yet. Open fullscreen (⛶) to get started.
          </div>
        ) : (
          <GraphCanvas
            nodes={nodes}
            edges={edges}
            selectedNodeId={selected_node_id}
            collapsedNodes={collapsed_nodes}
            onSelectNode={handleSelectNode}
            onToggleCollapse={toggleCollapse}
          />
        )}
      </div>

      {selectedNode && (
        <div style={{ padding: '6px 10px', borderTop: '1px solid var(--border)', background: 'var(--bg-panel)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <span style={{ width: 7, height: 7, borderRadius: 2, background: { topic: '#0e7490', category: '#15803d', leaf: '#b45309' }[selectedNode.node_type] ?? '#0e7490', display: 'inline-block' }} />
            <span style={{ color: 'var(--accent-ice)', fontWeight: 600, fontSize: 10 }}>{selectedNode.label}</span>
            <span style={{ fontSize: 8, background: 'var(--bg-hover)', padding: '1px 4px', borderRadius: 2, color: 'var(--text-dim)' }}>{selectedNode.node_type}</span>
            {Array.isArray(selectedNode.tags) && selectedNode.tags.slice(0, 3).map((tag) => (
              <span key={tag} style={{ fontSize: 8, background: 'var(--bg-hover)', padding: '0 4px', borderRadius: 2, color: 'var(--text-secondary)' }}>{tag}</span>
            ))}
          </div>
          <button onClick={() => selectNode(null)} style={{ background: 'none', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 11 }}>×</button>
        </div>
      )}
    </div>
  )
}
