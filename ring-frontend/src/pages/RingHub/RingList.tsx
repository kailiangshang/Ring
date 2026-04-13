import { Link } from 'react-router-dom'
import { Badge } from '../../components/ui/Badge'
import { EmptyState } from '../../components/ui/EmptyState'
import type { Ring } from '../../types'
import './RingHub.css'

interface RingListProps {
  rings: Ring[]
  on_select: (id: string) => void
  on_create: () => void
}

function format_relative_time(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export function RingList({ rings, on_select, on_create }: RingListProps) {
  if (rings.length === 0) {
    return (
      <EmptyState
        icon="⊕"
        title="No rings yet"
        description="Create your first Ring to get started with collaborative knowledge management."
        action_label="Create Ring"
        on_action={on_create}
      />
    )
  }

  return (
    <div className="ring-hub-grid">
      {rings.map((ring) => (
        <div
          key={ring.id}
          className="ring-card"
          onClick={() => on_select(ring.id)}
        >
          <div className="ring-card-header">
            <span className="ring-card-dot" />
            <span className="ring-card-name">{ring.name}</span>
          </div>
          <div className="ring-card-desc">
            {ring.description || 'No description'}
          </div>
          {(ring.member_count != null || ring.graph_node_count != null || ring.last_active_at) && (
            <div className="ring-card-stats">
              {ring.member_count != null && <span>👥 {ring.member_count}</span>}
              {ring.graph_node_count != null && <span>◉ {ring.graph_node_count}</span>}
              {ring.last_active_at && <span>{format_relative_time(ring.last_active_at)}</span>}
            </div>
          )}
          <div className="ring-card-divider" />
          <div className="ring-card-footer">
            <span className="ring-card-meta">{ring.status}</span>
            {ring.status === 'blueprint_pending' ? (
              <Link to={`/ring/${ring.id}/blueprint`} className="ring-card-cta" onClick={(e) => e.stopPropagation()}>
                设置蓝图 →
              </Link>
            ) : (
              <Badge status={ring.status}>{ring.status}</Badge>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
