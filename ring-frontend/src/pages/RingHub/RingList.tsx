import type { RingListItem } from '../../types'

interface RingListProps {
  rings: RingListItem[]
  on_select: (id: string) => void
}

export function RingList({ rings, on_select }: RingListProps) {
  if (rings.length === 0) {
    return <p>No rings yet. Create one to get started.</p>
  }

  return (
    <div>
      {rings.map((ring) => (
        <div
          key={ring.id}
          onClick={() => on_select(ring.id)}
          style={{
            border: '1px solid #ccc',
            padding: 12,
            marginBottom: 8,
            cursor: 'pointer',
          }}
        >
          <h3>{ring.name}</h3>
          <p>
            Members: {ring.member_count} | Nodes: {ring.graph_node_count} | Role: {ring.role}
          </p>
          <p>Last activity: {ring.last_activity_at}</p>
        </div>
      ))}
    </div>
  )
}
