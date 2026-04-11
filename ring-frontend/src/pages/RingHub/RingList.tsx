import { Badge } from '../../components/ui/Badge'
import { EmptyState } from '../../components/ui/EmptyState'
import type { Ring } from '../../types'
import './RingHub.css'

interface RingListProps {
  rings: Ring[]
  on_select: (id: string) => void
}

export function RingList({ rings, on_select }: RingListProps) {
  if (rings.length === 0) {
    return (
      <EmptyState
        icon="⊕"
        title="No rings yet"
        description="Create your first Ring to get started with collaborative knowledge management."
        action_label="Create Ring"
        on_action={undefined}
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
          <div className="ring-card-activity">
            Created: {new Date(ring.created_at).toLocaleDateString()}
          </div>
          <div className="ring-card-divider" />
          <div className="ring-card-footer">
            <span className="ring-card-meta">{ring.status}</span>
            <Badge status={ring.status}>{ring.status}</Badge>
          </div>
        </div>
      ))}
    </div>
  )
}
