import { useMemo, memo } from 'react'
import type { GraphNode } from '../../types/graph'

const NODE_COLORS: Record<string, string> = {
  topic: '#0891B2',
  category: '#22c55e',
  leaf: '#f59e0b',
}

interface TreeNode {
  node: GraphNode
  children: TreeNode[]
}

interface NodeTreeListProps {
  nodes: GraphNode[]
  selectedNodeId: string | null
  collapsedNodes: Set<string>
  onSelectNode: (id: string | null) => void
  onToggleCollapse: (id: string) => void
  onExpandAll: () => void
  onCollapseAll: () => void
}

function buildTree(nodes: GraphNode[]): TreeNode[] {
  const map = new Map<string, TreeNode>()
  const roots: TreeNode[] = []

  for (const n of nodes) {
    map.set(n.id, { node: n, children: [] })
  }

  for (const n of nodes) {
    const treeNode = map.get(n.id)!
    if (n.parent_id && map.has(n.parent_id)) {
      map.get(n.parent_id)!.children.push(treeNode)
    } else {
      roots.push(treeNode)
    }
  }

  return roots
}

const TreeNodeRow = memo(function TreeNodeRow({
  treeNode,
  depth,
  selectedNodeId,
  collapsedNodes,
  onSelectNode,
  onToggleCollapse,
}: {
  treeNode: TreeNode
  depth: number
  selectedNodeId: string | null
  collapsedNodes: Set<string>
  onSelectNode: (id: string | null) => void
  onToggleCollapse: (id: string) => void
}) {
  const { node, children } = treeNode
  const is_selected = node.id === selectedNodeId
  const has_children = children.length > 0
  const is_collapsed = collapsedNodes.has(node.id)

  return (
    <>
      <div
        onClick={() => onSelectNode(is_selected ? null : node.id)}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          paddingLeft: 12 + depth * 16,
          paddingRight: 8,
          paddingTop: 3,
          paddingBottom: 3,
          cursor: 'pointer',
          background: is_selected ? 'var(--bg-active)' : 'transparent',
          borderRadius: 3,
          margin: '1px 4px',
          fontSize: 11,
          color: is_selected ? 'var(--accent-ice)' : 'var(--text-primary)',
        }}
        onMouseEnter={(e) => {
          if (!is_selected) (e.currentTarget as HTMLDivElement).style.background = 'var(--bg-hover)'
        }}
        onMouseLeave={(e) => {
          if (!is_selected) (e.currentTarget as HTMLDivElement).style.background = 'transparent'
        }}
      >
        <span
          onClick={(e) => {
            e.stopPropagation()
            if (has_children) onToggleCollapse(node.id)
          }}
          style={{
            fontSize: 7,
            color: 'var(--text-dim)',
            cursor: has_children ? 'pointer' : 'default',
            width: 10,
            textAlign: 'center',
            flexShrink: 0,
            transition: 'transform 0.15s',
            transform: is_collapsed ? 'rotate(0deg)' : 'rotate(90deg)',
            userSelect: 'none',
            visibility: has_children ? 'visible' : 'hidden',
          }}
        >
          ▶
        </span>
        <span
          style={{
            width: 6,
            height: 6,
            borderRadius: '50%',
            background: NODE_COLORS[node.node_type] ?? '#0891B2',
            flexShrink: 0,
          }}
        />
        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flex: 1 }}>
          {node.label}
        </span>
        {Array.isArray(node.tags) && node.tags.length > 0 && (
          <span style={{ fontSize: 9, color: 'var(--text-dim)', flexShrink: 0 }}>
            {node.tags.length}
          </span>
        )}
      </div>
      {has_children && !is_collapsed && children.map((child) => (
        <TreeNodeRow
          key={child.node.id}
          treeNode={child}
          depth={depth + 1}
          selectedNodeId={selectedNodeId}
          collapsedNodes={collapsedNodes}
          onSelectNode={onSelectNode}
          onToggleCollapse={onToggleCollapse}
        />
      ))}
    </>
  )
})

export function NodeTreeList({
  nodes,
  selectedNodeId,
  collapsedNodes,
  onSelectNode,
  onToggleCollapse,
  onExpandAll,
  onCollapseAll,
}: NodeTreeListProps) {
  const tree = useMemo(() => buildTree(nodes), [nodes])

  if (nodes.length === 0) {
    return (
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
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{
        display: 'flex',
        gap: 4,
        padding: '4px 8px',
        borderBottom: '1px solid var(--border)',
      }}>
        <button
          onClick={onExpandAll}
          style={{
            fontSize: 9,
            padding: '2px 8px',
            borderRadius: 3,
            border: '1px solid var(--border)',
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
          }}
        >
          Expand All
        </button>
        <button
          onClick={onCollapseAll}
          style={{
            fontSize: 9,
            padding: '2px 8px',
            borderRadius: 3,
            border: '1px solid var(--border)',
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            cursor: 'pointer',
          }}
        >
          Collapse All
        </button>
      </div>
      <div style={{ flex: 1, overflowY: 'auto', padding: '4px 0' }}>
        {tree.map((rootNode) => (
          <TreeNodeRow
            key={rootNode.node.id}
            treeNode={rootNode}
            depth={0}
            selectedNodeId={selectedNodeId}
            collapsedNodes={collapsedNodes}
            onSelectNode={onSelectNode}
            onToggleCollapse={onToggleCollapse}
          />
        ))}
      </div>
    </div>
  )
}
