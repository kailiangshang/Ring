import { useEffect, useState, useMemo, useCallback } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { exportRingGraph } from '../../services/api'
import { api } from '../../services/api'
import { GraphCanvas } from './GraphCanvas'
import { NodeTreeList } from './NodeTreeList'
import type { EdgeRelation } from '../../types/graph'

interface DocRef {
  path: string
  title: string
  type: string
}

type ViewMode = 'canvas' | 'tree'

const EDGE_RELATIONS: { value: EdgeRelation; label: string }[] = [
  { value: 'related_to', label: 'related_to' },
  { value: 'depends_on', label: 'depends_on' },
  { value: 'derives_from', label: 'derives_from' },
  { value: 'contradicts', label: 'contradicts' },
]

function getNodeDocRefs(metadata: Record<string, unknown>): DocRef[] {
  const refs = metadata?.doc_refs
  if (!Array.isArray(refs)) return []
  return refs as DocRef[]
}

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
  const expandAll = useGraphStore((s) => s.expandAll)
  const collapseAll = useGraphStore((s) => s.collapseAll)

  const graphs = useGraphStore((s) => s.graphs)
  const createGraph = useGraphStore((s) => s.createGraph)
  const switchGraph = useGraphStore((s) => s.switchGraph)
  const graph_id = useGraphStore((s) => s.graph_id)

  const [newNodeLabel, setNewLabel] = useState('')
  const [selectedTags, setSelectedTags] = useState<Set<string>>(new Set())
  const [newGraphName, setNewGraphName] = useState('')
  const [showNewGraph, setShowNewGraph] = useState(false)
  const [exportMsg, setExportMsg] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('canvas')
  const [multiSelected, setMultiSelected] = useState<string[]>([])
  const [edgeRelation, setEdgeRelation] = useState<EdgeRelation>('related_to')
  const [edgeLabel, setEdgeLabel] = useState('')
  const [showDocRefForm, setShowDocRefForm] = useState(false)
  const [newDocRef, setNewDocRef] = useState<DocRef>({ path: '', title: '', type: 'archive' })

  useEffect(() => {
    if (active_ring_id) {
      fetchGraph(active_ring_id)
    }
  }, [active_ring_id, fetchGraph])

  const allTags = useMemo(() => {
    const tags = new Set<string>()
    nodes.forEach((n) => n.tags.forEach((t) => tags.add(t)))
    return Array.from(tags).sort()
  }, [nodes])

  const filteredNodes = useMemo(() => {
    if (selectedTags.size === 0) return nodes
    return nodes.filter((n) => n.tags.some((t) => selectedTags.has(t)))
  }, [nodes, selectedTags])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)

  const selectedDocRefs = useMemo(() => {
    if (!selectedNode) return []
    return getNodeDocRefs(selectedNode.metadata)
  }, [selectedNode])

  const parentNodeIds = useMemo(() => {
    return nodes.filter((n) => nodes.some((c) => c.parent_id === n.id)).map((n) => n.id)
  }, [nodes])

  const handleSelectNode = (nodeId: string | null, shiftKey: boolean = false) => {
    if (nodeId === null) {
      selectNode(null)
      setMultiSelected([])
      return
    }
    if (shiftKey) {
      setMultiSelected((prev) => {
        if (prev.includes(nodeId)) {
          return prev.filter((id) => id !== nodeId)
        }
        if (prev.length >= 2) {
          return [prev[1], nodeId]
        }
        return [...prev, nodeId]
      })
      selectNode(nodeId)
    } else {
      setMultiSelected([nodeId])
      selectNode(nodeId)
    }
  }

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

  const handleAddDocRef = useCallback(async () => {
    if (!active_ring_id || !selectedNode) return
    if (!newDocRef.path.trim() || !newDocRef.title.trim()) return
    const currentRefs = getNodeDocRefs(selectedNode.metadata)
    const updatedRefs = [...currentRefs, { ...newDocRef }]
    const currentMetadata = typeof selectedNode.metadata === 'object' && selectedNode.metadata !== null
      ? { ...selectedNode.metadata } as Record<string, unknown>
      : {}
    currentMetadata.doc_refs = updatedRefs
    try {
      await api.put(
        `/rings/${active_ring_id}/graph/nodes/${selectedNode.id}`,
        { metadata: currentMetadata },
      )
      await fetchGraph(active_ring_id)
      setNewDocRef({ path: '', title: '', type: 'archive' })
      setShowDocRefForm(false)
    } catch (e) {
      console.error('update doc refs failed:', e)
    }
  }, [active_ring_id, selectedNode, newDocRef, fetchGraph])

  const handleRemoveDocRef = useCallback(async (index: number) => {
    if (!active_ring_id || !selectedNode) return
    const currentRefs = getNodeDocRefs(selectedNode.metadata)
    const updatedRefs = currentRefs.filter((_, i) => i !== index)
    const currentMetadata = typeof selectedNode.metadata === 'object' && selectedNode.metadata !== null
      ? { ...selectedNode.metadata } as Record<string, unknown>
      : {}
    currentMetadata.doc_refs = updatedRefs
    try {
      await api.put(
        `/rings/${active_ring_id}/graph/nodes/${selectedNode.id}`,
        { metadata: currentMetadata },
      )
      await fetchGraph(active_ring_id)
    } catch (e) {
      console.error('remove doc ref failed:', e)
    }
  }, [active_ring_id, selectedNode, fetchGraph])

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
        <div style={{ display: 'flex', gap: 4, marginBottom: 6 }}>
          {(['canvas', 'tree'] as const).map((m) => (
            <button
              key={m}
              onClick={() => setViewMode(m)}
              style={{
                flex: 1,
                fontSize: 10,
                padding: '3px 0',
                borderRadius: 3,
                border: `1px solid ${viewMode === m ? 'var(--accent-cyan)' : 'var(--border)'}`,
                background: viewMode === m ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: viewMode === m ? 'var(--bg-base)' : 'var(--text-secondary)',
                cursor: 'pointer',
                fontWeight: viewMode === m ? 700 : 400,
                textTransform: 'capitalize',
              }}
            >
              {m}
            </button>
          ))}
        </div>
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
            onClick={async () => {
              if (!active_ring_id) return
              try {
                await exportRingGraph(active_ring_id)
                setExportMsg('Exported!')
              } catch { setExportMsg('Export failed') }
              setTimeout(() => setExportMsg(null), 2000)
            }}
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
            {exportMsg ?? 'Export'}
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
        {filteredNodes.length === 0 && nodes.length === 0 ? (
          <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100%',
            color: 'var(--text-dim)',
            fontSize: 12,
            textAlign: 'center',
            padding: 24,
          }}>
            No nodes yet. Type a label above and click +Node to get started.
          </div>
        ) : viewMode === 'canvas' ? (
          <GraphCanvas
            nodes={filteredNodes}
            edges={edges}
            selectedNodeId={selected_node_id}
            collapsedNodes={collapsed_nodes}
            onSelectNode={handleSelectNode}
            onToggleCollapse={toggleCollapse}
          />
        ) : (
          <NodeTreeList
            nodes={filteredNodes}
            selectedNodeId={selected_node_id}
            collapsedNodes={collapsed_nodes}
            onSelectNode={(id) => handleSelectNode(id)}
            onToggleCollapse={toggleCollapse}
            onExpandAll={() => expandAll(parentNodeIds)}
            onCollapseAll={() => collapseAll(parentNodeIds)}
          />
        )}
      </div>

      {selectedNode && (
        <div
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            background: 'var(--bg-panel)',
            fontSize: 11,
            maxHeight: 200,
            overflowY: 'auto',
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
                  if (active_ring_id && window.confirm(`Delete node "${selectedNode.label}"? This cannot be undone.`)) {
                    deleteNode(active_ring_id, selectedNode.id)
                  }
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
          {selectedDocRefs.length > 0 && (
            <div style={{ marginTop: 6 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', marginBottom: 3, fontWeight: 700 }}>
                关联文档
              </div>
              {selectedDocRefs.map((ref, i) => (
                <div
                  key={i}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                    marginBottom: 2,
                  }}
                >
                  <span style={{
                    fontSize: 9,
                    padding: '1px 4px',
                    borderRadius: 2,
                    background: ref.type === 'archive' ? 'rgba(34,211,238,0.15)' : 'rgba(167,139,250,0.15)',
                    color: 'var(--text-secondary)',
                  }}>
                    {ref.type}
                  </span>
                  <span style={{ fontSize: 10, color: 'var(--text-primary)' }}>
                    {ref.title}
                  </span>
                  <span style={{ fontSize: 9, color: 'var(--text-dim)' }}>
                    {ref.path}
                  </span>
                  <button
                    onClick={() => handleRemoveDocRef(i)}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--accent-amber)',
                      cursor: 'pointer',
                      fontSize: 9,
                      padding: '0 2px',
                    }}
                  >
                    移除
                  </button>
                </div>
              ))}
            </div>
          )}
          {showDocRefForm && (
            <div style={{ marginTop: 6, display: 'flex', gap: 4, alignItems: 'center', flexWrap: 'wrap' }}>
              <input
                value={newDocRef.path}
                onChange={(e) => setNewDocRef((prev) => ({ ...prev, path: e.target.value }))}
                placeholder="path..."
                style={{
                  flex: 1,
                  minWidth: 60,
                  background: 'var(--bg-input)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '2px 6px',
                  color: 'var(--text-primary)',
                  fontSize: 10,
                  outline: 'none',
                }}
              />
              <input
                value={newDocRef.title}
                onChange={(e) => setNewDocRef((prev) => ({ ...prev, title: e.target.value }))}
                placeholder="title..."
                style={{
                  flex: 1,
                  minWidth: 60,
                  background: 'var(--bg-input)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '2px 6px',
                  color: 'var(--text-primary)',
                  fontSize: 10,
                  outline: 'none',
                }}
              />
              <select
                value={newDocRef.type}
                onChange={(e) => setNewDocRef((prev) => ({ ...prev, type: e.target.value }))}
                style={{
                  background: 'var(--bg-input)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '2px 4px',
                  color: 'var(--text-primary)',
                  fontSize: 10,
                }}
              >
                <option value="archive">archive</option>
                <option value="upload">upload</option>
              </select>
              <button
                onClick={handleAddDocRef}
                disabled={!newDocRef.path.trim() || !newDocRef.title.trim()}
                style={{
                  fontSize: 9,
                  background: 'var(--accent-cyan)',
                  color: 'var(--bg-base)',
                  border: 'none',
                  borderRadius: 3,
                  padding: '2px 8px',
                  cursor: newDocRef.path.trim() && newDocRef.title.trim() ? 'pointer' : 'default',
                  opacity: newDocRef.path.trim() && newDocRef.title.trim() ? 1 : 0.4,
                }}
              >
                保存
              </button>
              <button
                onClick={() => { setShowDocRefForm(false); setNewDocRef({ path: '', title: '', type: 'archive' }) }}
                style={{
                  fontSize: 9,
                  background: 'none',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '2px 6px',
                  color: 'var(--text-secondary)',
                  cursor: 'pointer',
                }}
              >
                取消
              </button>
            </div>
          )}
          {!showDocRefForm && (
            <button
              onClick={() => setShowDocRefForm(true)}
              style={{
                marginTop: 4,
                fontSize: 9,
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '2px 8px',
                cursor: 'pointer',
              }}
            >
              添加文档
            </button>
          )}
        </div>
      )}

      {multiSelected.length === 2 && (() => {
        const src = nodes.find((n) => n.id === multiSelected[0])
        const tgt = nodes.find((n) => n.id === multiSelected[1])
        if (!src || !tgt) return null
        const existingEdge = edges.find(
          (e) =>
            (e.source_id === src.id && e.target_id === tgt.id) ||
            (e.source_id === tgt.id && e.target_id === src.id),
        )
        return (
          <div
            style={{
              padding: '8px 12px',
              borderTop: '1px solid var(--border)',
              background: 'var(--bg-panel)',
              fontSize: 11,
            }}
          >
            <div style={{ marginBottom: 6, fontWeight: 700, color: 'var(--accent-cyan)', fontSize: 11 }}>
              创建关联
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6, flexWrap: 'wrap' }}>
              <span style={{ fontSize: 10, background: 'var(--bg-hover)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-ice)' }}>
                {src.label}
              </span>
              <span style={{ color: 'var(--text-dim)', fontSize: 10 }}>→</span>
              <span style={{ fontSize: 10, background: 'var(--bg-hover)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-ice)' }}>
                {tgt.label}
              </span>
            </div>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center', marginBottom: 6 }}>
              <select
                value={edgeRelation}
                onChange={(e) => setEdgeRelation(e.target.value as EdgeRelation)}
                style={{
                  background: 'var(--bg-input)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 6px',
                  color: 'var(--text-primary)',
                  fontSize: 10,
                  fontFamily: 'inherit',
                }}
              >
                {EDGE_RELATIONS.map((r) => (
                  <option key={r.value} value={r.value}>{r.label}</option>
                ))}
              </select>
              <input
                value={edgeLabel}
                onChange={(e) => setEdgeLabel(e.target.value)}
                placeholder="label (optional)"
                style={{
                  flex: 1,
                  background: 'var(--bg-input)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 6px',
                  color: 'var(--text-primary)',
                  fontSize: 10,
                  fontFamily: 'inherit',
                  outline: 'none',
                }}
              />
            </div>
            <button
              onClick={() => {
                if (!active_ring_id) return
                if (existingEdge) {
                  setMultiSelected([])
                  return
                }
                useGraphStore.getState().createEdge(active_ring_id, src.id, tgt.id, edgeRelation)
                setMultiSelected([])
                setEdgeLabel('')
              }}
              disabled={!!existingEdge}
              style={{
                background: existingEdge ? 'var(--bg-hover)' : 'var(--accent-cyan)',
                color: existingEdge ? 'var(--text-dim)' : 'var(--bg-base)',
                border: 'none',
                borderRadius: 3,
                padding: '4px 12px',
                fontSize: 10,
                fontWeight: 700,
                cursor: existingEdge ? 'default' : 'pointer',
                width: '100%',
              }}
            >
              {existingEdge ? '已关联' : '创建'}
            </button>
            <button
              onClick={() => setMultiSelected([])}
              style={{
                background: 'none',
                border: 'none',
                color: 'var(--text-dim)',
                cursor: 'pointer',
                fontSize: 9,
                padding: '2px 0',
                marginTop: 4,
                width: '100%',
                textAlign: 'center',
              }}
            >
              取消选择
            </button>
          </div>
        )
      })()}
    </div>
  )
}
