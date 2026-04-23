import { useEffect, useState, useMemo } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { exportRingGraph } from '../../services/api'
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
  const collapsed_nodes = useGraphStore((s) => s.collapsed_nodes)
  const toggleCollapse = useGraphStore((s) => s.toggleCollapse)

  const graphs = useGraphStore((s) => s.graphs)
  const createGraph = useGraphStore((s) => s.createGraph)
  const switchGraph = useGraphStore((s) => s.switchGraph)
  const graph_id = useGraphStore((s) => s.graph_id)

  const [newNodeLabel, setNewLabel] = useState('')
  const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set())
  const [newGraphName, setNewGraphName] = useState('')
  const [showNewGraph, setShowNewGraph] = useState(false)

  useEffect(() => {
    if (active_ring_id) {
      fetchGraph(active_ring_id)
    }
  }, [active_ring_id, fetchGraph])

  // Extract all unique tags
  const allTags = useMemo(() => {
    const tags = new Set<string>()
    nodes.forEach((n) => n.tags.forEach((t) => tags.add(t)))
    return Array.from(tags).sort()
  }, [nodes])

  // Filter nodes by selected tags
  const filteredNodes = useMemo(() => {
    if (selectedTags.size === 0) return nodes
    return nodes.filter((n) => n.tags.some((t) => selectedTags.has(t)))
  }, [nodes, selectedTags])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)

  const handleCreateNode = () => {
    if (!newNodeLabel.trim() || !active_ring_id) return
    createNode(active_ring_id, newNodeLabel.trim())
    setNewLabel('')
  }

  const toggleTag = (tag: string) => {
    const newTags = new Set(selectedTags)
    if (newTags.has(tag)) {
      newTags.delete(tag)
    } else {
      newTags.add(tag)
    }
    setSelectedTags(newTags)
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
        {graphs.length > 1 && (
          <div style={{ marginBottom: 6, display: 'flex', gap: 4, flexWrap: 'wrap', alignItems: 'center' }}>
            {graphs.map((g) => (
              <button
                key={g.id}
                onClick={() => active_ring_id && switchGraph(active_ring_id, g.id)}
                style={{
                  fontSize: 10,
                  padding: '2px 8px',
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
          </div>
        )}
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
          <button
            onClick={() => active_ring_id && exportRingGraph(active_ring_id)}
            style={{
              background: 'var(--bg-hover)',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '4px 10px',
              fontSize: 11,
              cursor: 'pointer',
            }}
          >
            Export
          </button>
          {graphs.length < 3 && (
            <button
              onClick={() => setShowNewGraph(!showNewGraph)}
              style={{
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '4px 10px',
                fontSize: 11,
                cursor: 'pointer',
              }}
            >
              +Graph
            </button>
          )}
        </div>
        {showNewGraph && (
          <div style={{ marginTop: 6, display: 'flex', gap: 4 }}>
            <input
              value={newGraphName}
              onChange={(e) => setNewGraphName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && newGraphName.trim() && active_ring_id) {
                  createGraph(active_ring_id, newGraphName.trim())
                  setNewGraphName('')
                  setShowNewGraph(false)
                }
              }}
              placeholder="graph name..."
              style={{
                flex: 1,
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '4px 8px',
                color: 'var(--text-primary)',
                fontSize: 11,
                outline: 'none',
              }}
            />
            <button
              onClick={() => {
                if (newGraphName.trim() && active_ring_id) {
                  createGraph(active_ring_id, newGraphName.trim())
                  setNewGraphName('')
                  setShowNewGraph(false)
                }
              }}
              style={{
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 4,
                padding: '4px 10px',
                fontSize: 11,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              Create
            </button>
          </div>
        )}

        {/* Tag filter */}
        {allTags.length > 0 && (
          <div style={{ marginTop: 6, display: 'flex', gap: 4, flexWrap: 'wrap', alignItems: 'center' }}>
            <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>Filter:</span>
            {allTags.map((tag) => (
              <button
                key={tag}
                onClick={() => toggleTag(tag)}
                style={{
                  fontSize: 9,
                  padding: '1px 6px',
                  borderRadius: 2,
                  border: '1px solid var(--border)',
                  background: selectedTags.has(tag) ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: selectedTags.has(tag) ? 'var(--bg-base)' : 'var(--text-secondary)',
                  cursor: 'pointer',
                }}
              >
                {tag}
              </button>
            ))}
            {selectedTags.size > 0 && (
              <button
                onClick={() => setSelectedTags(new Set())}
                style={{
                  fontSize: 9,
                  padding: '1px 6px',
                  borderRadius: 2,
                  border: 'none',
                  background: 'none',
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                }}
              >
                Clear
              </button>
            )}
          </div>
        )}

        <div style={{ marginTop: 4, fontSize: 10, color: 'var(--text-dim)' }}>
          {filteredNodes.length} / {nodes.length} nodes · {edges.length} edges
          {selectedTags.size > 0 && ` · ${selectedTags.size} tag filter${selectedTags.size > 1 ? 's' : ''} active`}
        </div>
      </div>

      <div style={{ flex: 1, minHeight: 0 }}>
        <GraphCanvas
          nodes={filteredNodes}
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
