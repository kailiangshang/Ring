import { useState } from 'react'
import type { GraphNode } from '../../types'

const TYPE_ICONS: Record<string, string> = {
  concept: '\u{1F4A1}',
  category: '\u{1F4C1}',
  document: '\u{1F4C4}',
  event: '\u26A1',
  person: '\u{1F464}',
  task: '\u2705',
}

interface NodeTreeProps {
  nodes: GraphNode[]
  selected_node_id: string | null
  on_select: (node_id: string) => void
}

interface TreeNode {
  node: GraphNode
  children: TreeNode[]
}

function build_tree(nodes: GraphNode[]): TreeNode[] {
  const map = new Map<string, TreeNode>()
  for (const n of nodes) {
    map.set(n.id, { node: n, children: [] })
  }
  const roots: TreeNode[] = []
  for (const n of nodes) {
    const tn = map.get(n.id)!
    if (n.parent_id && map.has(n.parent_id)) {
      map.get(n.parent_id)!.children.push(tn)
    } else {
      roots.push(tn)
    }
  }
  return roots
}

function TreeNodeItem({
  tree_node,
  selected_node_id,
  on_select,
  depth,
}: {
  tree_node: TreeNode
  selected_node_id: string | null
  on_select: (node_id: string) => void
  depth: number
}) {
  const [expanded, set_expanded] = useState(true)
  const is_selected = tree_node.node.id === selected_node_id
  const has_children = tree_node.children.length > 0
  const icon = TYPE_ICONS[tree_node.node.node_type] || '\u{1F4CB}'

  return (
    <div>
      <div
        onClick={() => on_select(tree_node.node.id)}
        style={{
          padding: '4px 8px',
          paddingLeft: depth * 16 + 8,
          cursor: 'pointer',
          background: is_selected ? '#e0e7ff' : 'transparent',
          fontWeight: is_selected ? 600 : 400,
          display: 'flex',
          alignItems: 'center',
          gap: 4,
          userSelect: 'none',
        }}
        onMouseEnter={(e) => {
          if (!is_selected) (e.currentTarget.style.background = '#f3f4f6')
        }}
        onMouseLeave={(e) => {
          if (!is_selected) (e.currentTarget.style.background = 'transparent')
        }}
      >
        {has_children && (
          <span
            onClick={(e) => {
              e.stopPropagation()
              set_expanded(!expanded)
            }}
            style={{ width: 16, textAlign: 'center', fontSize: 10 }}
          >
            {expanded ? '\u25BC' : '\u25B6'}
          </span>
        )}
        {!has_children && <span style={{ width: 16 }} />}
        <span>{icon}</span>
        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {tree_node.node.label}
        </span>
      </div>
      {expanded &&
        tree_node.children.map((child) => (
          <TreeNodeItem
            key={child.node.id}
            tree_node={child}
            selected_node_id={selected_node_id}
            on_select={on_select}
            depth={depth + 1}
          />
        ))}
    </div>
  )
}

export function NodeTree({ nodes, selected_node_id, on_select }: NodeTreeProps) {
  const tree = build_tree(nodes)

  return (
    <div data-testid="node-tree" style={{ overflow: 'auto', height: '100%' }}>
      {tree.map((tn) => (
        <TreeNodeItem
          key={tn.node.id}
          tree_node={tn}
          selected_node_id={selected_node_id}
          on_select={on_select}
          depth={0}
        />
      ))}
    </div>
  )
}
