import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useGitStore } from '../../stores/gitStore'
import { Tabs } from '../../components/ui/Tabs'
import { Badge } from '../../components/ui/Badge'
import { EmptyState } from '../../components/ui/EmptyState'
import { ArchiveQueueBar } from '../../components/archive/ArchiveQueueBar'
import './PrPages.css'

const STATE_TABS = [
  { key: 'opened', label: 'Opened' },
  { key: 'merged', label: 'Merged' },
  { key: 'closed', label: 'Closed' },
]

export function PrList() {
  const { ringId } = useParams<{ ringId: string }>()
  const navigate = useNavigate()
  const { prs, loading, error, load_prs } = useGitStore()
  const [state_filter, set_state_filter] = useState('opened')

  useEffect(() => {
    if (ringId) load_prs(ringId, state_filter)
  }, [ringId, state_filter, load_prs])

  return (
    <div className="pr-list">
      <div className="pr-list-header">
        <h2 className="pr-list-title">PRs</h2>
      </div>

      <Tabs tabs={STATE_TABS} active_key={state_filter} on_change={set_state_filter} />

      <ArchiveQueueBar ring_id={ringId!} />

      {error && <p className="setup-error" role="alert">{error}</p>}
      {loading && <p>Loading...</p>}

      {!loading && prs.length === 0 && (
        <EmptyState
          icon="📋"
          title="No PRs found"
          description="No pull requests match the selected filter."
        />
      )}

      {!loading &&
        prs.map((pr) => (
          <div
            key={pr.pr_id}
            className="pr-row"
            onClick={() => navigate(`/ring/${ringId}/prs/${pr.pr_id}`)}
          >
            <Badge status={pr.state}>{pr.state}</Badge>
            <span className="pr-row-title">{pr.title}</span>
            <span className="pr-row-author">{pr.author}</span>
          </div>
        ))}
    </div>
  )
}
