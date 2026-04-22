import { useEffect, useState } from 'react'
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
  const createNode = useGraphStore((s) => s.createNode)
  const deleteNode = useGraphStore((s) => s.deleteNode)
  const selectNode = useGraphStore((s) => s.selectNode)

  const [newNodeLabel, setNewLabel] = useState('')
  const collapsed_nodes = useGraphStore((s) => s.collapsed_nodes)
  const toggleCollapse = useGraphStore((s) => s.toggleCollapse)

  useEffect(() => {
    if (active_ring_id) {
      fetchGraph(active_ring_id)
    }
  }, [active_ring_id, fetchGraph])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)

  const handleCreateNode = () => {
    if (!newNodeLabel.trim() || !active_ring_id) return
    createNode(active_ring_id, newNodeLabel.trim())
    setNewLabel('')
  }

  if (loading) {
    return (
      <div style={{ padding: 16, color: 'var(--text-dim)', fontSize: 12 }}>
        Loading graph...
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 12px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
          <input
            value={newNodeLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCreateNode()
            }}
            placeholder="node label..."
            style={{
              flex: 1,
              background: 'var(--bg-input)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '4px 8px',
              color: 'var(--text-primary)',
              fontSize: 11,
              fontFamily: 'inherit',
              outline: 'none',
            }}
          />
          <button
            onClick={handleCreateNode}
            disabled={!newNodeLabel.trim()}
            style={{
              background: 'var(--accent-cyan)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '4px 12px',
              fontSize: 11,
              fontWeight: 700,
              cursor: newNodeLabel.trim() ? 'pointer' : 'default',
              opacity: newNodeLabel.trim() ? 1 : 0.4,
            }}
          >
            +Node
          </button>
        </div>
        <div style={{ marginTop: 4, fontSize: 10, color: 'var(--text-dim)' }}>
          {nodes.length} nodes · {edges.length} edges
        </div>
      </div>

      <div style={{ flex: 1, minHeight: 0 }}>
        <GraphCanvas
          nodes={nodes}
          edges={edges}
          selectedNodeId={selected_node_id}
          collapsedNodes={collapsed_nodes}
          onSelectNode={selectNode}
          onToggleCollapse={toggleCollapse}
        />
      </div>

      {selectedNode && (
        <div
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            background: 'var(--bg-panel)',
            fontSize: 11,
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ color: 'var(--accent-ice)', fontWeight: 700 }}>
              {selectedNode.label}
            </span>
            <div style={{ display: 'flex', gap: 4 }}>
              <span
                style={{
                  fontSize: 9,
                  background: 'var(--bg-hover)',
                  padding: '2px 6px',
                  borderRadius: 2,
                  color: 'var(--text-dim)',
                }}
              >
                {selectedNode.node_type}
              </span>
              <button
                onClick={() => {
                  if (active_ring_id) deleteNode(active_ring_id, selectedNode.id)
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                  fontSize: 10,
                  padding: '0 4px',
                }}
              >
                ×
              </button>
            </div>
          </div>
          {selectedNode.tags.length > 0 && (
            <div style={{ marginTop: 4, display: 'flex', gap: 4, flexWrap: 'wrap' }}>
              {selectedNode.tags.map((tag) => (
                <span
                  key={tag}
                  style={{
                    fontSize: 9,
                    background: 'var(--bg-hover)',
                    padding: '1px 6px',
                    borderRadius: 2,
                    color: 'var(--text-secondary)',
                  }}
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
