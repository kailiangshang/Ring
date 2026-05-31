import { useEffect, useState, useMemo, useCallback } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { exportRingGraph } from '../../services/api'
import { api } from '../../services/api'
import { GraphCanvas } from './GraphCanvas'
import { NodeTreeList } from './NodeTreeList'
import type { EdgeRelation } from '../../types/graph'
import { ConfirmModal } from '../common/ConfirmModal'

interface DocRef {
  path: string
  title: string
  type: string
}

type ViewMode = 'canvas' | 'tree'
type DraftEndpoint = 'source' | 'target'

interface RelationDraft {
  sourceId: string | null
  targetId: string | null
  picking: DraftEndpoint | null
}

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
  const selected_edge_id = useGraphStore((s) => s.selected_edge_id)
  const fetchGraph = useGraphStore((s) => s.fetchGraph)
  const createNode = useGraphStore((s) => s.createNode)
  const deleteNode = useGraphStore((s) => s.deleteNode)
  const updateEdge = useGraphStore((s) => s.updateEdge)
  const deleteEdge = useGraphStore((s) => s.deleteEdge)
  const selectNode = useGraphStore((s) => s.selectNode)
  const selectEdge = useGraphStore((s) => s.selectEdge)
  const collapsed_nodes = useGraphStore((s) => s.collapsed_nodes)
  const toggleCollapse = useGraphStore((s) => s.toggleCollapse)
  const expandAll = useGraphStore((s) => s.expandAll)
  const collapseAll = useGraphStore((s) => s.collapseAll)
  const toggleFloat = useGraphStore((s) => s.toggleFloat)

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
  const [relationDraft, setRelationDraft] = useState<RelationDraft>({ sourceId: null, targetId: null, picking: null })
  const [edgeRelation, setEdgeRelation] = useState<EdgeRelation>('related_to')
  const [edgeLabel, setEdgeLabel] = useState('')
  const [showDocRefForm, setShowDocRefForm] = useState(false)
  const [newDocRef, setNewDocRef] = useState<DocRef>({ path: '', title: '', type: 'archive' })
  const [confirmDialog, setConfirmDialog] = useState<{ title: string; message: string; action: () => void; variant?: 'danger' | 'default' } | null>(null)

  useEffect(() => {
    if (active_ring_id) {
      fetchGraph(active_ring_id)
    }
  }, [active_ring_id, fetchGraph])

  const allTags = useMemo(() => {
    const tags = new Set<string>()
    nodes.forEach((n) => {
      if (Array.isArray(n.tags)) n.tags.forEach((t) => tags.add(t))
    })
    return Array.from(tags).sort()
  }, [nodes])

  const filteredNodes = useMemo(() => {
    if (selectedTags.size === 0) return nodes
    return nodes.filter((n) => Array.isArray(n.tags) && n.tags.some((t) => selectedTags.has(t)))
  }, [nodes, selectedTags])

  const selectableNodes = useMemo(() => {
    return [...nodes].sort((a, b) => a.label.localeCompare(b.label))
  }, [nodes])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)
  const selectedEdge = edges.find((e) => e.id === selected_edge_id)
  const draftSourceNode = nodes.find((n) => n.id === relationDraft.sourceId) ?? null
  const draftTargetNode = nodes.find((n) => n.id === relationDraft.targetId) ?? null
  const draftExistingEdge = relationDraft.sourceId && relationDraft.targetId
    ? edges.find(
      (e) =>
        (e.source_id === relationDraft.sourceId && e.target_id === relationDraft.targetId) ||
        (e.source_id === relationDraft.targetId && e.target_id === relationDraft.sourceId),
    ) ?? null
    : null

  const selectedDocRefs = useMemo(() => {
    if (!selectedNode) return []
    return getNodeDocRefs(selectedNode.metadata)
  }, [selectedNode])

  const parentNodeIds = useMemo(() => {
    return nodes.filter((n) => nodes.some((c) => c.parent_id === n.id)).map((n) => n.id)
  }, [nodes])


  useEffect(() => {
    if (!selectedEdge) return
    setEdgeRelation(selectedEdge.relation)
    setEdgeLabel(selectedEdge.label ?? '')
  }, [selectedEdge])

  const clearRelationDraft = useCallback(() => {
    setRelationDraft({ sourceId: null, targetId: null, picking: null })
    setEdgeLabel('')
  }, [])

  const beginRelationFromNode = useCallback((nodeId: string) => {
    setRelationDraft({ sourceId: nodeId, targetId: null, picking: 'target' })
    setEdgeRelation('related_to')
    setEdgeLabel('')
    selectEdge(null)
  }, [selectEdge])

  const handleSelectNode = (nodeId: string | null, _shiftKey: boolean = false) => {
    if (nodeId === null) {
      selectNode(null)
      return
    }

    if (relationDraft.picking) {
      setRelationDraft((prev) => {
        if (prev.picking === 'source') {
          return {
            sourceId: nodeId,
            targetId: prev.targetId,
            picking: prev.targetId ? null : 'target',
          }
        }
        return {
          sourceId: prev.sourceId,
          targetId: nodeId,
          picking: null,
        }
      })
      selectNode(nodeId)
      return
    }

    if (relationDraft.sourceId && !relationDraft.targetId && nodeId !== relationDraft.sourceId) {
      setRelationDraft((prev) => ({
        ...prev,
        targetId: nodeId,
        picking: null,
      }))
      selectNode(nodeId)
      return
    }

    selectNode(nodeId)
  }

  const handleSelectEdge = (edgeId: string | null) => {
    setRelationDraft({ sourceId: null, targetId: null, picking: null })
    selectEdge(edgeId)
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
              if (e.key === 'Enter' && !e.nativeEvent.isComposing) handleCreateNode()
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
            onClick={() => toggleFloat()}
            style={{
              background: 'var(--bg-hover)',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '4px 10px',
              fontSize: 11,
              cursor: 'pointer',
            }}
            title="Open floating graph"
          >
            Float
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
                if (e.key === 'Enter' && !e.nativeEvent.isComposing && newGraphName.trim() && active_ring_id) {
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
          {filteredNodes.length} / {nodes.length} nodes | {edges.length} edges
          {selectedTags.size > 0 && ` | ${selectedTags.size} tag filter${selectedTags.size > 1 ? 's' : ''} active`}
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
            selectedEdgeId={selected_edge_id}
            relationDraftSourceId={relationDraft.sourceId}
            relationDraftTargetId={relationDraft.targetId}
            collapsedNodes={collapsed_nodes}
            onSelectNode={handleSelectNode}
            onSelectEdge={handleSelectEdge}
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
                onClick={() => beginRelationFromNode(selectedNode.id)}
                style={{
                  background: relationDraft.sourceId === selectedNode.id ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  color: relationDraft.sourceId === selectedNode.id ? 'var(--bg-base)' : 'var(--text-secondary)',
                  cursor: 'pointer',
                  fontSize: 10,
                  padding: '0 6px',
                  borderRadius: 3,
                }}
              >
                Link
              </button>
              <button
                onClick={() => {
                  if (active_ring_id) {
                    setConfirmDialog({
                      title: 'Delete Node',
                      message: `Delete node "${selectedNode.label}"? This cannot be undone.`,
                      variant: 'danger' as const,
                      action: () => { deleteNode(active_ring_id, selectedNode.id) },
                    })
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
                Delete
              </button>
            </div>
          </div>
          {Array.isArray(selectedNode.tags) && selectedNode.tags.length > 0 && (
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
                Linked Docs
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
                    Remove
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
                Save
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
                Cancel
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
              Add Doc
            </button>
          )}
        </div>
      )}

      {selectedEdge && (
        <div
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            background: 'var(--bg-panel)',
            fontSize: 11,
          }}
        >
          <div style={{ marginBottom: 6, fontWeight: 700, color: 'var(--accent-cyan)', fontSize: 11 }}>
            Edit Relation
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6, flexWrap: 'wrap' }}>
            <span style={{ fontSize: 10, background: 'var(--bg-hover)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-ice)' }}>
              {nodes.find((n) => n.id === selectedEdge.source_id)?.label ?? selectedEdge.source_id}
            </span>
            <span style={{ color: 'var(--text-dim)', fontSize: 10 }}>{'->'}</span>
            <span style={{ fontSize: 10, background: 'var(--bg-hover)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-ice)' }}>
              {nodes.find((n) => n.id === selectedEdge.target_id)?.label ?? selectedEdge.target_id}
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
          <div style={{ display: 'flex', gap: 6 }}>
            <button
              onClick={() => {
                if (!active_ring_id) return
                updateEdge(active_ring_id, selectedEdge.id, { relation: edgeRelation, label: edgeLabel })
              }}
              style={{
                flex: 1,
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 3,
                padding: '4px 12px',
                fontSize: 10,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              Save Relation
            </button>
            <button
              onClick={() => {
                if (!active_ring_id) return
                setConfirmDialog({
                  title: 'Delete Relation',
                  message: 'Delete this relation? This cannot be undone.',
                  variant: 'danger',
                  action: () => deleteEdge(active_ring_id, selectedEdge.id),
                })
              }}
              style={{
                background: 'var(--bg-hover)',
                color: 'var(--accent-amber)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '4px 10px',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              Delete
            </button>
          </div>
        </div>
      )}

      {(relationDraft.sourceId || relationDraft.targetId || relationDraft.picking) && (
        <div
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            background: 'var(--bg-panel)',
            fontSize: 11,
          }}
        >
          <div style={{ marginBottom: 6, fontWeight: 700, color: 'var(--accent-cyan)', fontSize: 11 }}>
            Create Relation
          </div>
          <div style={{ display: 'grid', gap: 6, marginBottom: 8 }}>
            {(['source', 'target'] as DraftEndpoint[]).map((endpoint) => {
              const node = endpoint === 'source' ? draftSourceNode : draftTargetNode
              const active = relationDraft.picking === endpoint
              const accent = endpoint === 'source' ? 'var(--accent-cyan)' : 'var(--accent-amber)'
              return (
                <div key={endpoint} style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                  <button
                    onClick={() => setRelationDraft((prev) => ({ ...prev, picking: endpoint }))}
                    style={{
                      minWidth: 56,
                      background: active ? accent : 'var(--bg-hover)',
                      color: active ? 'var(--bg-base)' : 'var(--text-secondary)',
                      border: '1px solid var(--border)',
                      borderRadius: 3,
                      padding: '3px 8px',
                      fontSize: 10,
                      cursor: 'pointer',
                      textTransform: 'capitalize',
                    }}
                  >
                    {endpoint}
                  </button>
                  <button
                    onClick={() => setRelationDraft((prev) => ({ ...prev, picking: endpoint }))}
                    title={`Pick ${endpoint} from the graph`}
                    style={{
                      fontSize: 10,
                      padding: '3px 8px',
                      borderRadius: 999,
                      border: `1px solid ${active ? accent : 'var(--border)'}`,
                      color: node ? 'var(--text-primary)' : 'var(--text-dim)',
                      background: endpoint === 'source' ? 'rgba(34,211,238,0.12)' : 'rgba(245,158,11,0.12)',
                      cursor: 'pointer',
                      fontFamily: 'inherit',
                    }}
                  >
                    {node?.label ?? `Choose ${endpoint}`}
                  </button>
                  <select
                    value={node?.id ?? ''}
                    onChange={(e) => {
                      const value = e.target.value || null
                      setRelationDraft((prev) => ({
                        ...prev,
                        sourceId: endpoint === 'source' ? value : prev.sourceId,
                        targetId: endpoint === 'target' ? value : prev.targetId,
                        picking: null,
                      }))
                    }}
                    style={{
                      minWidth: 140,
                      background: 'var(--bg-input)',
                      border: `1px solid ${active ? accent : 'var(--border)'}`,
                      borderRadius: 3,
                      padding: '3px 6px',
                      color: 'var(--text-primary)',
                      fontSize: 10,
                      fontFamily: 'inherit',
                    }}
                  >
                    <option value="">{`Choose ${endpoint}`}</option>
                    {selectableNodes.map((candidate) => (
                      <option key={candidate.id} value={candidate.id}>{candidate.label}</option>
                    ))}
                  </select>
                  {selectedNode && (
                    <button
                      onClick={() => setRelationDraft((prev) => ({
                        ...prev,
                        sourceId: endpoint === 'source' ? selectedNode.id : prev.sourceId,
                        targetId: endpoint === 'target' ? selectedNode.id : prev.targetId,
                        picking: null,
                      }))}
                      style={{
                        background: 'none',
                        border: '1px solid var(--border)',
                        color: 'var(--text-secondary)',
                        borderRadius: 3,
                        padding: '3px 6px',
                        fontSize: 10,
                        cursor: 'pointer',
                      }}
                    >
                      Use Selected
                    </button>
                  )}
                  {node && (
                    <button
                      onClick={() => setRelationDraft((prev) => ({
                        ...prev,
                        sourceId: endpoint === 'source' ? null : prev.sourceId,
                        targetId: endpoint === 'target' ? null : prev.targetId,
                        picking: endpoint,
                      }))}
                      style={{
                        background: 'none',
                        border: '1px solid var(--border)',
                        color: 'var(--text-secondary)',
                        borderRadius: 3,
                        padding: '3px 6px',
                        fontSize: 10,
                        cursor: 'pointer',
                      }}
                    >
                      Clear
                    </button>
                  )}
                </div>
              )
            })}
          </div>

          <div style={{ color: 'var(--text-dim)', fontSize: 10, marginBottom: 8 }}>
            {relationDraft.picking
              ? `Click a node in the graph to set the ${relationDraft.picking}.`
              : 'Choose Source or Target to replace either endpoint.'}
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 6, flexWrap: 'wrap' }}>
            <span style={{ fontSize: 10, background: 'rgba(34,211,238,0.12)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-ice)' }}>
              {draftSourceNode?.label ?? 'Unset source'}
            </span>
            <span style={{ color: 'var(--text-dim)', fontSize: 10 }}>{'->'}</span>
            <span style={{ fontSize: 10, background: 'rgba(245,158,11,0.12)', padding: '2px 6px', borderRadius: 2, color: 'var(--accent-amber)' }}>
              {draftTargetNode?.label ?? 'Unset target'}
            </span>
            <button
              onClick={() => setRelationDraft((prev) => ({
                sourceId: prev.targetId,
                targetId: prev.sourceId,
                picking: prev.picking,
              }))}
              disabled={!relationDraft.sourceId || !relationDraft.targetId}
              style={{
                background: 'none',
                border: '1px solid var(--border)',
                color: !relationDraft.sourceId || !relationDraft.targetId ? 'var(--text-dim)' : 'var(--text-secondary)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                cursor: !relationDraft.sourceId || !relationDraft.targetId ? 'default' : 'pointer',
              }}
            >
              Swap
            </button>
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

          {relationDraft.sourceId === relationDraft.targetId && relationDraft.sourceId && (
            <div style={{ color: 'var(--accent-amber)', fontSize: 10, marginBottom: 6 }}>
              Source and target must be different nodes.
            </div>
          )}

          <button
            onClick={() => {
              if (!active_ring_id || !draftSourceNode || !draftTargetNode || draftSourceNode.id === draftTargetNode.id) return
              if (draftExistingEdge) {
                selectEdge(draftExistingEdge.id)
                clearRelationDraft()
                return
              }
              useGraphStore.getState().createEdge(active_ring_id, draftSourceNode.id, draftTargetNode.id, edgeRelation, edgeLabel)
              clearRelationDraft()
            }}
            disabled={!draftSourceNode || !draftTargetNode || draftSourceNode.id === draftTargetNode.id}
            style={{
              background: draftExistingEdge ? 'var(--bg-hover)' : 'var(--accent-cyan)',
              color: draftExistingEdge ? 'var(--text-secondary)' : 'var(--bg-base)',
              border: 'none',
              borderRadius: 3,
              padding: '4px 12px',
              fontSize: 10,
              fontWeight: 700,
              cursor: !draftSourceNode || !draftTargetNode || draftSourceNode.id === draftTargetNode.id ? 'default' : 'pointer',
              width: '100%',
              opacity: !draftSourceNode || !draftTargetNode || draftSourceNode.id === draftTargetNode.id ? 0.55 : 1,
            }}
          >
            {draftExistingEdge ? 'Open Existing Relation' : 'Create Relation'}
          </button>
          <button
            onClick={clearRelationDraft}
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
            Clear Selection
          </button>
        </div>
      )}
      <ConfirmModal
        open={confirmDialog !== null}
        title={confirmDialog?.title ?? ''}
        message={confirmDialog?.message ?? ''}
        variant={confirmDialog?.variant}
        on_confirm={() => { confirmDialog?.action(); setConfirmDialog(null) }}
        on_cancel={() => setConfirmDialog(null)}
      />
    </div>
  )
}
