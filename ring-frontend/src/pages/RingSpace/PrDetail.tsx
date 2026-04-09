import { useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useGitStore } from '../../stores/gitStore'
import { Button } from '../../components/ui/Button'
import { DiffView } from '../../components/git/DiffView'
import './PrPages.css'

export function PrDetail() {
  const { ringId, prId } = useParams<{ ringId: string; prId: string }>()
  const navigate = useNavigate()
  const { current_pr, loading, error, load_pr_detail, merge_pr, reject_pr } =
    useGitStore()

  const pr_id_num = prId ? parseInt(prId, 10) : 0

  useEffect(() => {
    if (ringId && prId) load_pr_detail(ringId, pr_id_num)
  }, [ringId, prId, pr_id_num, load_pr_detail])

  const handle_merge = async () => {
    if (ringId) {
      await merge_pr(ringId, pr_id_num)
      navigate(`/ring/${ringId}/prs`)
    }
  }

  const handle_reject = async () => {
    if (ringId) {
      await reject_pr(ringId, pr_id_num)
      navigate(`/ring/${ringId}/prs`)
    }
  }

  if (loading) return <div className="pr-detail"><p>Loading...</p></div>
  if (error) return <div className="pr-detail"><p className="setup-error" role="alert">{error}</p></div>
  if (!current_pr) return null

  return (
    <div className="pr-detail">
      <button
        className="pr-detail-back"
        onClick={() => navigate(`/ring/${ringId}/prs`)}
      >
        &larr; Back to PRs
      </button>

      <div className="pr-detail-title-row">
        <h2>#{current_pr.pr_id}</h2>
        <h3>{current_pr.title}</h3>
        <span className="pr-detail-author">by {current_pr.author}</span>
      </div>

      <div className="pr-detail-actions">
        <Button onClick={handle_merge}>Merge</Button>
        <Button variant="danger" onClick={handle_reject}>Reject</Button>
      </div>

      <DiffView changes={current_pr.changes} />
    </div>
  )
}
