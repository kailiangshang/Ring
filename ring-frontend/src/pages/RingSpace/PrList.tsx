import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useGitStore } from '../../stores/gitStore'

export function PrList() {
  const { ringId } = useParams<{ ringId: string }>()
  const navigate = useNavigate()
  const { prs, loading, error, load_prs } = useGitStore()
  const [state_filter, set_state_filter] = useState('opened')

  useEffect(() => {
    if (ringId) load_prs(ringId, state_filter)
  }, [ringId, state_filter, load_prs])

  const state_badge_color = (state: string) => {
    switch (state) {
      case 'opened':
        return '#28a745'
      case 'merged':
        return '#6f42c1'
      case 'closed':
        return '#cb2431'
      default:
        return '#888'
    }
  }

  return (
    <div style={{ padding: '1.5rem', maxWidth: '800px', margin: '0 auto' }}>
      <h2>PRs</h2>

      <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
        {['opened', 'merged', 'closed'].map((s) => (
          <button
            key={s}
            onClick={() => set_state_filter(s)}
            style={{
              padding: '0.4rem 0.8rem',
              border: '1px solid #ddd',
              borderRadius: '4px',
              background: state_filter === s ? '#0366d6' : '#fff',
              color: state_filter === s ? '#fff' : '#333',
              cursor: 'pointer',
            }}
          >
            {s}
          </button>
        ))}
      </div>

      {error && <p style={{ color: 'red' }}>{error}</p>}
      {loading && <p>Loading...</p>}

      {!loading && prs.length === 0 && (
        <p style={{ color: '#888' }}>No PRs found</p>
      )}

      {!loading &&
        prs.map((pr) => (
          <div
            key={pr.pr_id}
            onClick={() => navigate(`/ring/${ringId}/prs/${pr.pr_id}`)}
            style={{
              padding: '0.75rem',
              border: '1px solid #e0e0e0',
              borderRadius: '4px',
              marginBottom: '0.5rem',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '0.75rem',
            }}
          >
            <span
              style={{
                padding: '2px 8px',
                borderRadius: '3px',
                fontSize: '0.75rem',
                fontWeight: 600,
                color: '#fff',
                background: state_badge_color(pr.state),
              }}
            >
              {pr.state}
            </span>
            <span style={{ fontWeight: 500 }}>{pr.title}</span>
            <span style={{ marginLeft: 'auto', color: '#888', fontSize: '0.85rem' }}>
              {pr.author}
            </span>
          </div>
        ))}
    </div>
  )
}
