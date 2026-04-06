import { useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useGitStore } from '../../stores/gitStore'
import { DiffView } from '../../components/git/DiffView'

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

  if (loading) return <p style={{ padding: '1.5rem' }}>Loading...</p>
  if (error) return <p style={{ padding: '1.5rem', color: 'red' }}>{error}</p>
  if (!current_pr) return null

  return (
    <div style={{ padding: '1.5rem', maxWidth: '900px', margin: '0 auto' }}>
      <button
        onClick={() => navigate(`/ring/${ringId}/prs`)}
        style={{
          background: 'none',
          border: 'none',
          color: '#0366d6',
          cursor: 'pointer',
          marginBottom: '1rem',
          fontSize: '0.9rem',
        }}
      >
        &larr; Back to PRs
      </button>

      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '1rem' }}>
        <h2>#{current_pr.pr_id}</h2>
        <h3 style={{ margin: 0 }}>{current_pr.title}</h3>
        <span style={{ color: '#888' }}>by {current_pr.author}</span>
      </div>

      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1.5rem' }}>
        <button
          onClick={handle_merge}
          style={{
            padding: '0.5rem 1rem',
            background: '#28a745',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Merge
        </button>
        <button
          onClick={handle_reject}
          style={{
            padding: '0.5rem 1rem',
            background: '#dc3545',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Reject
        </button>
      </div>

      <DiffView changes={current_pr.changes} />
    </div>
  )
}
