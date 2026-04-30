import { useState, useEffect, useCallback } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { useSessionStore } from '../../stores/session-store'
import { useAppStore } from '../../stores/app-store'
import { usePanelStore } from '../../stores/panel-store'
import type { Session, SessionPhase } from '../../types/session'

const phase_color: Record<SessionPhase, string> = {
  material_prep: 'var(--accent-amber)',
  discussion: 'var(--accent-green)',
  summary: 'var(--accent-cyan)',
  closed: 'var(--text-dim)',
}

function SessionRow({ session, is_active }: { session: Session; is_active: boolean }) {
  const openPanel = usePanelStore((s) => s.open)

  return (
    <div
      onClick={(e) => {
        e.stopPropagation()
        openPanel('session')
      }}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        padding: '3px 8px 3px 36px',
        cursor: 'pointer',
        background: is_active ? 'var(--bg-active)' : 'transparent',
        borderRadius: 3,
        margin: '1px 6px',
        fontSize: 11,
        color: is_active ? 'var(--accent-ice)' : 'var(--text-secondary)',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
      }}
      title={`${session.title} (${session.phase})`}
    >
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: '50%',
          background: phase_color[session.phase],
          flexShrink: 0,
        }}
      />
      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{session.title}</span>
      <span style={{ marginLeft: 'auto', fontSize: 9, color: 'var(--text-dim)', flexShrink: 0 }}>
        {session.skill.replace('_', ' ')}
      </span>
    </div>
  )
}

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const loading = useRingStore((s) => s.loading)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const createRing = useRingStore((s) => s.createRing)
  const deleteRing = useRingStore((s) => s.deleteRing)
  const setContext = useAppStore((s) => s.setContext)
  const selectRing = useRingStore((s) => s.selectRing)
  const openPanel = usePanelStore((s) => s.open)
  const sessions_by_ring = useSessionStore((s) => s.sessions_by_ring)
  const fetchSessionsForSidebar = useSessionStore((s) => s.fetchSessionsForSidebar)
  const active_session = useSessionStore((s) => s.active_session)
  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')
  const [createError, setCreateError] = useState<string | null>(null)
  const [startBlueprint, setStartBlueprint] = useState(true)
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})
  const [storageMode, setStorageMode] = useState<'local' | 'gitlab'>('local')
  const [gitlabRepoUrl, setGitlabRepoUrl] = useState('')
  const [deletingRing, setDeletingRing] = useState<string | null>(null)

  const toggle_expand = useCallback((ring_id: string) => {
    setExpanded((prev) => ({ ...prev, [ring_id]: !prev[ring_id] }))
  }, [])

  useEffect(() => {
    if (!active_ring_id) return
    fetchSessionsForSidebar(active_ring_id)
  }, [active_ring_id, fetchSessionsForSidebar])

  useEffect(() => {
    if (!active_ring_id) return
    setExpanded((prev) => ({ ...prev, [active_ring_id]: true }))
  }, [active_ring_id])

  const handleCreate = async () => {
    if (!newName.trim()) return
    if (storageMode === 'gitlab' && !gitlabRepoUrl.trim()) return
    setCreateError(null)
    const ring_id = await createRing({
      name: newName.trim(),
      role_description: `You are a ${newName.trim()} assistant`,
      storage_mode: storageMode,
      gitlab_repo_url: storageMode === 'gitlab' ? gitlabRepoUrl.trim() : undefined,
    })
    if (ring_id) {
      setNewName('')
      setCreating(false)
      setStorageMode('local')
      setGitlabRepoUrl('')
      selectRing(ring_id)
      setContext('ring')
      if (startBlueprint) {
        openPanel('blueprint')
      }
    } else {
      setCreateError('Failed to create ring. Name may already exist.')
    }
  }

  return (
    <div style={{ padding: '8px 0' }}>
      {loading && rings.length === 0 && (
        <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11 }}>Loading rings...</div>
      )}
      {!loading && rings.length === 0 && (
        <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11, textAlign: 'center' }}>
          No rings yet. Create one below.
        </div>
      )}
      {rings.map((ring) => {
        const is_expanded = expanded[ring.id]
        const sessions = sessions_by_ring[ring.id] ?? []
        const is_active_ring = ring.id === active_ring_id

        return (
          <div key={ring.id}>
            <div
              onClick={() => {
                selectRing(ring.id)
                setContext('ring')
                if (!is_expanded) toggle_expand(ring.id)
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                padding: '8px 12px',
                cursor: 'pointer',
                background: is_active_ring ? 'var(--bg-active)' : 'transparent',
                borderRadius: 4,
                margin: '2px 6px',
              }}
              onMouseEnter={(e) => {
                if (!is_active_ring) (e.currentTarget as HTMLDivElement).style.background = 'var(--bg-hover)'
              }}
              onMouseLeave={(e) => {
                if (!is_active_ring) (e.currentTarget as HTMLDivElement).style.background = 'transparent'
              }}
            >
              <span
                onClick={(e) => {
                  e.stopPropagation()
                  toggle_expand(ring.id)
                }}
                style={{
                  fontSize: 8,
                  color: 'var(--text-dim)',
                  cursor: 'pointer',
                  width: 10,
                  textAlign: 'center',
                  flexShrink: 0,
                  transition: 'transform 0.15s',
                  transform: is_expanded ? 'rotate(90deg)' : 'rotate(0deg)',
                  userSelect: 'none',
                }}
              >
                ▶
              </span>
              <span
                style={{
                  color: is_active_ring ? 'var(--accent-ice)' : 'var(--text-primary)',
                  fontWeight: is_active_ring ? 700 : 400,
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
              >
                {ring.name}
              </span>
              <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', fontSize: 11 }}>
                {ring.member_count}
              </span>
              {ring.has_active_session && (
                <span
                  title="Active session"
                  style={{
                    width: 8,
                    height: 8,
                    borderRadius: '50%',
                    background: 'var(--accent-green)',
                    flexShrink: 0,
                  }}
                />
              )}
              {deletingRing === ring.id ? (
                <span style={{ fontSize: 9, color: 'var(--accent-amber)' }}>Deleting...</span>
              ) : (
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    if (confirm(`Delete ring "${ring.name}"? This cannot be undone.`)) {
                      setDeletingRing(ring.id)
                      deleteRing(ring.id).then((ok) => {
                        setDeletingRing(null)
                        if (ok && active_ring_id === ring.id) {
                          selectRing(null)
                          setContext('super')
                        }
                      })
                    }
                  }}
                  title="Delete ring"
                  style={{
                    background: 'none',
                    border: 'none',
                    color: 'var(--text-dim)',
                    cursor: 'pointer',
                    fontSize: 11,
                    padding: '0 2px',
                    opacity: 0.3,
                    transition: 'opacity 0.15s',
                  }}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.opacity = '1'
                    ;(e.currentTarget as HTMLButtonElement).style.color = 'var(--accent-red)'
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLButtonElement).style.opacity = '0.3'
                    ;(e.currentTarget as HTMLButtonElement).style.color = 'var(--text-dim)'
                  }}
                >
                  🗑
                </button>
              )}
            </div>
            {is_expanded && sessions.length > 0 && (
              <div>
                {sessions.map((s) => (
                  <SessionRow
                    key={s.id}
                    session={s}
                    is_active={active_session?.id === s.id}
                  />
                ))}
              </div>
            )}
            {is_expanded && sessions.length === 0 && (
              <div style={{ padding: '2px 8px 2px 36px', fontSize: 10, color: 'var(--text-dim)' }}>
                No sessions
              </div>
            )}
          </div>
        )
      })}

      {creating ? (
        <div style={{ padding: '8px 12px' }}>
          <input
            autoFocus
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCreate()
              if (e.key === 'Escape') setCreating(false)
            }}
            placeholder="Ring name..."
            style={{
              width: '100%',
              background: 'var(--bg-input)',
              border: '1px solid var(--accent-cyan)',
              borderRadius: 3,
              padding: '5px 8px',
              color: 'var(--text-primary)',
              fontSize: 11,
              fontFamily: 'inherit',
              outline: 'none',
              marginBottom: 4,
            }}
          />
          {createError && (
            <div style={{ fontSize: 10, color: 'var(--accent-amber)', marginBottom: 4 }}>
              {createError}
            </div>
          )}
          <div style={{ display: 'flex', gap: 4, marginBottom: 6 }}>
            {(['local', 'gitlab'] as const).map((m) => (
              <button
                key={m}
                onClick={() => setStorageMode(m)}
                style={{
                  flex: 1,
                  background: storageMode === m ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: storageMode === m ? 'var(--bg-base)' : 'var(--text-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 0',
                  fontSize: 9,
                  cursor: 'pointer',
                  fontWeight: storageMode === m ? 700 : 400,
                  textTransform: 'capitalize',
                }}
              >
                {m}
              </button>
            ))}
          </div>
          {storageMode === 'gitlab' && (
            <input
              value={gitlabRepoUrl}
              onChange={(e) => setGitlabRepoUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreate()
                if (e.key === 'Escape') setCreating(false)
              }}
              placeholder="https://gitlab.company.com/group/project"
              style={{
                width: '100%',
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '5px 8px',
                color: 'var(--text-primary)',
                fontSize: 11,
                fontFamily: 'inherit',
                outline: 'none',
                marginBottom: 4,
              }}
            />
          )}
          <label style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            fontSize: 10,
            color: 'var(--text-secondary)',
            marginBottom: 6,
            cursor: 'pointer',
          }}>
            <input
              type="checkbox"
              checked={startBlueprint}
              onChange={(e) => setStartBlueprint(e.target.checked)}
              style={{ cursor: 'pointer' }}
            />
            Start with blueprint
          </label>
          <div style={{ display: 'flex', gap: 4 }}>
            <button
              onClick={handleCreate}
              style={{
                flex: 1,
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 3,
                padding: '4px 0',
                fontSize: 10,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              CREATE
            </button>
            <button
              onClick={() => setCreating(false)}
              style={{
                flex: 1,
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '4px 0',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              CANCEL
            </button>
          </div>
        </div>
      ) : (
        <div
          onClick={() => setCreating(true)}
          style={{
            margin: '4px 12px',
            padding: '6px 0',
            border: '1px solid var(--accent-cyan)',
            borderRadius: 3,
            textAlign: 'center',
            color: 'var(--accent-cyan)',
            fontSize: 10,
            cursor: 'pointer',
            fontWeight: 700,
          }}
        >
          + new ring
        </div>
      )}
      <div style={{ padding: '8px 12px', borderTop: '1px solid var(--border)', marginTop: 8 }}>
        <div style={{ fontSize: 9, color: 'var(--text-muted)', lineHeight: 1.4 }}>
          对话记录仅保存在当前设备
        </div>
      </div>
    </div>
  )
}
