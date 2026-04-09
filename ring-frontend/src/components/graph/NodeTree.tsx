import { useState } from 'react'
import type { GraphNode } from '../../types'
import './NodeTree.css'

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
  highlighted_node_id?: string | null
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
  highlighted_node_id,
  on_select,
  depth,
}: {
  tree_node: TreeNode
  selected_node_id: string | null
  highlighted_node_id?: string | null
  on_select: (node_id: string) => void
  depth: number
}) {
  const [expanded, set_expanded] = useState(true)
  const is_selected = tree_node.node.id === selected_node_id
  const is_highlighted = tree_node.node.id === highlighted_node_id
  const has_children = tree_node.children.length > 0
  const icon = TYPE_ICONS[tree_node.node.node_type] || '\u{1F4CB}'

  const cls = [
    'node-tree-item',
    is_selected && 'node-tree-item-selected',
    is_highlighted && !is_selected && 'node-tree-item-highlighted',
  ].filter(Boolean).join(' ')

  return (
    <div>
      <div
        className={cls}
        onClick={() => on_select(tree_node.node.id)}
        style={{ '--node-depth': depth } as React.CSSProperties}
      >
        {has_children && (
          <span
            className="node-tree-toggle"
            onClick={(e) => {
              e.stopPropagation()
              set_expanded(!expanded)
            }}
          >
            {expanded ? '\u25BC' : '\u25B6'}
          </span>
        )}
        {!has_children && <span className="node-tree-spacer" />}
        <span>{icon}</span>
        <span className="node-tree-label">{tree_node.node.label}</span>
      </div>
      {expanded &&
        tree_node.children.map((child) => (
          <TreeNodeItem
            key={child.node.id}
            tree_node={child}
            selected_node_id={selected_node_id}
            highlighted_node_id={highlighted_node_id}
            on_select={on_select}
            depth={depth + 1}
          />
        ))}
    </div>
  )
}

export function NodeTree({ nodes, selected_node_id, on_select, highlighted_node_id }: NodeTreeProps) {
  const tree = build_tree(nodes)

  return (
    <div className="node-tree" data-testid="node-tree">
      {tree.map((tn) => (
        <TreeNodeItem
          key={tn.node.id}
          tree_node={tn}
          selected_node_id={selected_node_id}
          highlighted_node_id={highlighted_node_id}
          on_select={on_select}
          depth={0}
        />
      ))}
    </div>
  )
}
