import { useEffect, useState } from 'react'
import { useParams, NavLink, useLocation } from 'react-router-dom'
import * as api from '../../api/client'
import { NodeTree } from '../graph/NodeTree'
import { useRightPanel } from './RingSpaceLayout'
import type { GraphNode } from '../../types'
import './RingSidebar.css'

interface RingSidebarProps { collapsed: boolean; on_toggle: () => void }

const NAV_ITEMS = [
  { path: '', label: 'Chat', icon: '💬' },
  { path: '/graph', label: 'Graph', icon: '◉' },
  { path: '/prs', label: 'PRs', icon: '📋' },
  { path: '/members', label: 'Members', icon: '👥' },
  { path: '/sessions', label: 'Sessions', icon: '🔍' },
]

export function RingSidebar({ collapsed, on_toggle }: RingSidebarProps) {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()
  const [nodes, set_nodes] = useState<GraphNode[]>([])
  const [selected_node_id, set_selected_node_id] = useState<string | null>(null)
  const { set_panel } = useRightPanel()

  useEffect(() => {
    if (!ringId) return
    api.list_graphs(ringId).then((graph_ids) => {
      if (graph_ids.length > 0) {
        api.get_graph(ringId, graph_ids[0]).then((detail) => set_nodes(detail.nodes))
      }
    }).catch(() => {})
  }, [ringId])

  const handle_node_select = (node_id: string) => {
    set_selected_node_id(node_id)
    const node = nodes.find((n) => n.id === node_id)
    if (node) {
      set_panel({ open: true, content: 'node_detail', data: node })
    }
  }

  if (!ringId) return null

  return (
    <div className={`ring-sidebar${collapsed ? ' ring-sidebar-collapsed' : ''}`}>
      <div className="ring-sidebar-tree">
        {!collapsed && nodes.length > 0 && (
          <NodeTree
            nodes={nodes}
            selected_node_id={selected_node_id}
            on_select={handle_node_select}
          />
        )}
        {!collapsed && nodes.length === 0 && (
          <div className="ring-sidebar-placeholder">暂无图谱节点</div>
        )}
      </div>
      <div className="ring-sidebar-divider" />
      <div className="ring-sidebar-nav">
        {NAV_ITEMS.map((item) => {
          const to = `/ring/${ringId}${item.path}`
          const is_active = item.path === ''
            ? location.pathname === `/ring/${ringId}` || location.pathname === `/ring/${ringId}/`
            : location.pathname.startsWith(to)
          return (
            <NavLink key={item.path} to={to} end={item.path === ''} className={`ring-sidebar-nav-item${is_active ? ' sidebar-active' : ''}`} title={collapsed ? item.label : undefined}>
              <span>{collapsed ? item.icon : item.label}</span>
            </NavLink>
          )
        })}
      </div>
      <button className="ring-sidebar-collapse-btn" onClick={on_toggle}>
        {collapsed ? '→' : '← 收起'}
      </button>
    </div>
  )
}
