import { useEffect, useState } from 'react'
import { useArchiveStore } from '../../stores/archive-store'
import { useRingStore } from '../../stores/ring-store'
import { getArchiveDiff, getGitLog, postGitRevert } from '../../services/api'
import type { ArchiveRecord } from '../../types/archive'

const STATUS_LABELS: Record<string, string> = {
  pending: 'pending',
  committed: 'committed',
  pushed: 'pushed',
  mr_opened: 'MR open',
  merged: 'merged',
  rejected: 'rejected',
}

const row: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
}

export function ArchivePanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const archives = useArchiveStore((s) => s.archives)
  const queue = useArchiveStore((s) => s.queue)
  const repoStatus = useArchiveStore((s) => s.repoStatus)
  const loading = useArchiveStore((s) => s.loading)
  const archiving = useArchiveStore((s) => s.archiving)
  const progress = useArchiveStore((s) => s.progress)
  const fetchArchives = useArchiveStore((s) => s.fetchArchives)
  const fetchQueue = useArchiveStore((s) => s.fetchQueue)
  const fetchRepoStatus = useArchiveStore((s) => s.fetchRepoStatus)
  const reviewArchive = useArchiveStore((s) => s.reviewArchive)
  const initRepo = useArchiveStore((s) => s.initRepo)

  const [selected, setSelected] = useState<ArchiveRecord | null>(null)
  const [diffData, setDiffData] = useState<Array<{ old_path: string; new_path: string; diff: string }> | null>(null)
  const [diffLoading, setDiffLoading] = useState(false)
  const [initLoading, setInitLoading] = useState(false)
  const [commits, setCommits] = useState<Array<{ sha: string; subject: string; author: string; date: string }>>([])
  const [commitsLoading, setCommitsLoading] = useState(false)

  const ringId = active_ring_id ?? ''

  useEffect(() => {
    if (!ringId) return
    fetchArchives(ringId)
    fetchQueue(ringId)
    fetchRepoStatus(ringId)
  }, [ringId])

  if (repoStatus && !repoStatus.initialized) {
    return (
      <div style={{ padding: 16 }}>
        <p style={{ color: 'var(--text-secondary)', fontSize: 12, marginBottom: 12 }}>
          Git repo not initialized
        </p>
        <button
          onClick={async () => { setInitLoading(true); await initRepo(ringId); setInitLoading(false) }}
          disabled={initLoading}
          style={{
            background: 'var(--accent-ice)',
            color: '#000',
            border: 'none',
            borderRadius: 4,
            padding: '6px 14px',
            fontSize: 12,
            fontWeight: 600,
            cursor: initLoading ? 'default' : 'pointer',
            opacity: initLoading ? 0.5 : 1,
          }}
        >
          {initLoading ? 'Initializing...' : 'Initialize repo'}
        </button>
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', fontSize: 12 }}>
      <div
        style={{
          ...row,
          padding: '8px 12px',
          borderBottom: '1px solid var(--border)',
          justifyContent: 'space-between',
        }}
      >
        <span style={{ fontWeight: 700, color: 'var(--accent-ice)', letterSpacing: '0.05em' }}>
          Archives
        </span>
        {archiving && (
          <span style={{ color: 'var(--text-muted)', fontSize: 11 }}>{progress}</span>
        )}
      </div>

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <div
          style={{
            width: 260,
            borderRight: '1px solid var(--border)',
            overflowY: 'auto',
            flexShrink: 0,
          }}
        >
          {loading ? (
            <div style={{ padding: 12, color: 'var(--text-muted)' }}>Loading...</div>
          ) : archives.length === 0 ? (
            <div style={{ padding: 12, color: 'var(--text-muted)' }}>No archives yet</div>
          ) : (
            archives.map((a) => (
              <div
                key={a.id}
                onClick={() => setSelected(a)}
                style={{
                  ...row,
                  padding: '8px 12px',
                  cursor: 'pointer',
                  background:
                    selected?.id === a.id ? 'var(--bg-hover)' : 'transparent',
                  borderBottom: '1px solid var(--border)',
                  gap: 6,
                }}
              >
                <span
                  style={{
                    fontSize: 10,
                    padding: '2px 6px',
                    borderRadius: 3,
                    background:
                      a.status === 'merged'
                        ? 'var(--accent-green, #2ea043)'
                        : a.status === 'rejected'
                          ? 'var(--accent-red, #da3633)'
                          : 'var(--bg-input)',
                    color:
                      a.status === 'merged' || a.status === 'rejected'
                        ? '#fff'
                        : 'var(--text-secondary)',
                    flexShrink: 0,
                  }}
                >
                  {STATUS_LABELS[a.status] || a.status}
                </span>
                <span
                  style={{
                    flex: 1,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                    color: 'var(--text-primary)',
                  }}
                >
                  {a.file_name}
                </span>
                <span style={{ color: 'var(--text-muted)', fontSize: 11, flexShrink: 0 }}>
                  {a.created_at.slice(0, 10)}
                </span>
              </div>
            ))
          )}
        </div>

        <div style={{ flex: 1, padding: 16, overflowY: 'auto' }}>
          {selected ? (
            <div>
              <h3
                style={{
                  margin: 0,
                  marginBottom: 12,
                  fontSize: 13,
                  color: 'var(--text-primary)',
                  fontWeight: 700,
                }}
              >
                {selected.file_name}
              </h3>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <span style={{ color: 'var(--text-secondary)' }}>
                  Status: {STATUS_LABELS[selected.status]}
                </span>
                <span style={{ color: 'var(--text-secondary)' }}>
                  By: {selected.archived_by}
                </span>
                {selected.commit_sha && (
                  <span style={{ color: 'var(--text-muted)', fontFamily: 'monospace' }}>
                    Commit: {selected.commit_sha.slice(0, 8)}
                  </span>
                )}
                {selected.merge_request_iid && (
                  <span style={{ color: 'var(--accent-ice)' }}>
                    MR !{selected.merge_request_iid}
                  </span>
                )}
                {selected.merge_request_iid && (
                  <button
                    onClick={async () => {
                      if (!active_ring_id) return
                      setDiffLoading(true)
                      try {
                        const res = await getArchiveDiff(active_ring_id, selected.id)
                        setDiffData(res.diffs)
                      } catch {
                        setDiffData([])
                      }
                      setDiffLoading(false)
                    }}
                    style={{
                      background: 'var(--bg-hover)',
                      border: '1px solid var(--border)',
                      borderRadius: 3,
                      padding: '3px 8px',
                      fontSize: 10,
                      color: 'var(--accent-cyan)',
                      cursor: 'pointer',
                      marginTop: 4,
                    }}
                  >
                    {diffLoading ? 'Loading...' : 'View Diff'}
                  </button>
                )}
                {diffData && diffData.length > 0 && (
                  <div style={{ marginTop: 12, borderTop: '1px solid var(--border)', paddingTop: 8 }}>
                    <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 8 }}>
                      Diff
                    </div>
                    {diffData.map((d, i) => (
                      <div key={i} style={{ marginBottom: 12 }}>
                        <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginBottom: 4 }}>
                          {d.new_path}
                        </div>
                        <pre style={{
                          background: 'var(--bg-base)',
                          border: '1px solid var(--border)',
                          borderRadius: 3,
                          padding: 6,
                          fontSize: 9,
                          overflow: 'auto',
                          maxHeight: 200,
                          color: 'var(--text-primary)',
                          margin: 0,
                        }}>
                          {d.diff}
                        </pre>
                      </div>
                    ))}
                  </div>
                )}
                {diffData && diffData.length === 0 && (
                  <div style={{ marginTop: 8, fontSize: 10, color: 'var(--text-dim)' }}>
                    No diff available
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div style={{ color: 'var(--text-muted)' }}>Select an archive to view details</div>
          )}
        </div>
      </div>

      {queue.length > 0 && (
        <div style={{ borderTop: '1px solid var(--border)', flexShrink: 0 }}>
          <div
            style={{
              padding: '8px 12px',
              fontWeight: 700,
              color: 'var(--accent-ice)',
              letterSpacing: '0.05em',
              fontSize: 11,
            }}
          >
            PR Review Queue ({queue.length})
          </div>
          {queue.map((mr) => (
            <div
              key={mr.id}
              style={{
                ...row,
                padding: '6px 12px',
                justifyContent: 'space-between',
                borderTop: '1px solid var(--border)',
              }}
            >
              <span style={{ color: 'var(--text-primary)' }}>{mr.file_name}</span>
              <div style={{ ...row, gap: 6 }}>
                <button
                  onClick={() => {
                    if (window.confirm(`Merge "${mr.file_name}"? This cannot be undone.`)) {
                      reviewArchive(ringId, mr.id, 'merge')
                    }
                  }}
                  style={{
                    background: 'var(--accent-green, #2ea043)',
                    color: '#fff',
                    border: 'none',
                    borderRadius: 4,
                    padding: '3px 10px',
                    fontSize: 11,
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  Merge
                </button>
                <button
                  onClick={() => {
                    if (window.confirm(`Reject "${mr.file_name}"?`)) {
                      reviewArchive(ringId, mr.id, 'reject')
                    }
                  }}
                  style={{
                    background: 'var(--accent-red, #da3633)',
                    color: '#fff',
                    border: 'none',
                    borderRadius: 4,
                    padding: '3px 10px',
                    fontSize: 11,
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  Reject
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      <div style={{ borderTop: '1px solid var(--border)', flexShrink: 0, maxHeight: 200, overflowY: 'auto' }}>
        <div
          style={{
            ...row,
            padding: '8px 12px',
            justifyContent: 'space-between',
            position: 'sticky',
            top: 0,
            background: 'var(--bg-panel)',
            zIndex: 1,
          }}
        >
          <span style={{ fontWeight: 700, color: 'var(--accent-ice)', letterSpacing: '0.05em', fontSize: 11 }}>
            Git History
          </span>
          <button
            onClick={async () => {
              if (!ringId) return
              setCommitsLoading(true)
              try {
                const res = await getGitLog(ringId)
                setCommits(res.commits)
              } catch { setCommits([]) }
              setCommitsLoading(false)
            }}
            style={{
              background: 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '2px 8px',
              fontSize: 9,
              color: 'var(--text-secondary)',
              cursor: 'pointer',
            }}
          >
            {commitsLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
        {commits.length === 0 && !commitsLoading && (
          <div style={{ padding: '6px 12px', color: 'var(--text-dim)', fontSize: 10 }}>
            Click Refresh to load commit history
          </div>
        )}
        {commits.map((c) => (
          <div
            key={c.sha}
            style={{
              ...row,
              padding: '4px 12px',
              borderTop: '1px solid var(--border)',
              gap: 6,
              alignItems: 'center',
            }}
          >
            <span style={{ fontFamily: 'monospace', fontSize: 10, color: 'var(--text-dim)', flexShrink: 0 }}>
              {c.sha.slice(0, 8)}
            </span>
            <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', color: 'var(--text-primary)', fontSize: 11 }}>
              {c.subject}
            </span>
            <button
              onClick={async () => {
                if (!ringId || !window.confirm(`Revert "${c.subject}"?`)) return
                try {
                  await postGitRevert(ringId, c.sha)
                  const res = await getGitLog(ringId)
                  setCommits(res.commits)
                } catch (e: any) {
                  alert(e?.message || 'Revert failed')
                }
              }}
              style={{
                background: 'var(--accent-red, #da3633)',
                color: '#fff',
                border: 'none',
                borderRadius: 3,
                padding: '1px 6px',
                fontSize: 9,
                cursor: 'pointer',
                flexShrink: 0,
              }}
            >
              Revert
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}
