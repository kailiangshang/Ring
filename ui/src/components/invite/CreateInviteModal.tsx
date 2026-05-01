import { useState, useEffect } from 'react'
import { Modal } from '../common/Modal'
import { useInviteStore } from '../../stores/invite-store'
import { useRingStore } from '../../stores/ring-store'
import type { InviteToken } from '../../types/invite'

export function CreateInviteModal() {
  const modal_open = useInviteStore((s) => s.modal_open)
  const close_modal = useInviteStore((s) => s.close_modal)
  const create_token = useInviteStore((s) => s.create_token)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const [link_type, set_link_type] = useState<'open' | 'audit'>('open')
  const [role, set_role] = useState('member')
  const [max_uses, set_max_uses] = useState(1)
  const [max_members, set_max_members] = useState<string>('')
  const [expires_hours, set_expires_hours] = useState(24)
  const [created_token, set_created_token] = useState<InviteToken | null>(null)
  const [creating, set_creating] = useState(false)
  const [copied, set_copied] = useState(false)
  const [localIp, setLocalIp] = useState<string | null>(null)

  useEffect(() => {
    fetch('/api/network/info')
      .then((res) => res.json())
      .then((data) => {
        if (data.local_ip) {
          setLocalIp(data.local_ip)
        }
      })
      .catch(() => {
        // fallback to window.location.hostname
        setLocalIp(window.location.hostname)
      })
  }, [])

  const handle_create = async () => {
    if (!active_ring_id) return
    set_creating(true)
    try {
      const token = await create_token(active_ring_id, {
        type: link_type,
        role,
        max_uses,
        max_members: max_members ? parseInt(max_members, 10) : null,
        expires_in_hours: expires_hours,
      })
      set_created_token(token)
    } catch {
      // ignore
    } finally {
      set_creating(false)
    }
  }

  const getInviteLink = () => {
    const host = localIp || window.location.hostname
    return `http://${host}:7420/ring/join?token=${created_token!.token}`
  }

  const handle_copy = async () => {
    if (!created_token) return
    const link = getInviteLink()
    await navigator.clipboard.writeText(link)
    set_copied(true)
    setTimeout(() => set_copied(false), 2000)
  }

  const handle_another = () => {
    set_created_token(null)
    set_copied(false)
  }

  const handle_done = () => {
    set_created_token(null)
    set_copied(false)
    set_link_type('open')
    set_role('member')
    set_max_uses(1)
    set_max_members('')
    set_expires_hours(24)
    close_modal()
  }

  return (
    <Modal open={modal_open} on_close={handle_done}>
      {created_token ? (
        <div>
          <div style={{ padding: '14px 20px', borderBottom: '1px solid var(--border)', background: 'var(--bg-sidebar)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ color: 'var(--accent-green)', fontSize: 12, fontWeight: 600, letterSpacing: 1 }}>✓ LINK CREATED</span>
            <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 16, padding: '2px 6px', borderRadius: 3 }} onClick={handle_done}>×</span>
          </div>
          <div style={{ padding: 20 }}>
            <div style={{ background: 'var(--bg-active)', border: '1px solid var(--accent-cyan)', borderRadius: 4, padding: 12, marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--accent-cyan)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Invite Link</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <code style={{ flex: 1, fontSize: 10, color: 'var(--accent-ice)', wordBreak: 'break-all', lineHeight: 1.5 }}>
                  {getInviteLink()}
                  {localIp === null && (
                    <span style={{ color: 'var(--accent-amber)', fontSize: 9, marginLeft: 4 }}>
                      (⚠️ using current hostname)
                    </span>
                  )}
                </code>
                <div
                  style={{ padding: '6px 12px', background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderRadius: 3, fontSize: 9, fontWeight: 700, letterSpacing: 1, cursor: 'pointer', whiteSpace: 'nowrap' }}
                  onClick={handle_copy}
                >
                  {copied ? 'COPIED' : 'COPY'}
                </div>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 16, marginBottom: 16, fontSize: 10, color: 'var(--text-secondary)' }}>
              <div><span style={{ color: 'var(--text-dim)' }}>Type:</span> {created_token.type}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Role:</span> {created_token.role}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Uses:</span> {created_token.use_count}/{created_token.max_uses}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Expires:</span> {expires_hours}h</div>
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              <div style={{ flex: 1, padding: 8, border: '1px solid var(--border)', borderRadius: 4, textAlign: 'center', fontSize: 10, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={handle_another}>CREATE ANOTHER</div>
              <div style={{ flex: 1, padding: 8, border: '1px solid var(--border)', borderRadius: 4, textAlign: 'center', fontSize: 10, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={handle_done}>DONE</div>
            </div>
          </div>
        </div>
      ) : (
        <div>
          <div style={{ padding: '14px 20px', borderBottom: '1px solid var(--border)', background: 'var(--bg-sidebar)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ color: 'var(--accent-ice)', fontSize: 12, fontWeight: 600, letterSpacing: 1 }}>🔗 CREATE INVITE</span>
            <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 16, padding: '2px 6px', borderRadius: 3 }} onClick={handle_done}>×</span>
          </div>
          <div style={{ padding: 20 }}>
            <div style={{ marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 8 }}>Link Type</div>
              <div style={{ display: 'flex', gap: 8 }}>
                <div
                  style={{ flex: 1, padding: '10px 14px', border: `1px solid ${link_type === 'open' ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, background: link_type === 'open' ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                  onClick={() => set_link_type('open')}
                >
                  <div style={{ fontSize: 11, color: link_type === 'open' ? 'var(--accent-cyan)' : 'var(--text-secondary)', fontWeight: 600 }}>Open Link</div>
                  <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 2 }}>Join directly, no approval</div>
                </div>
                <div
                  style={{ flex: 1, padding: '10px 14px', border: `1px solid ${link_type === 'audit' ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, background: link_type === 'audit' ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                  onClick={() => set_link_type('audit')}
                >
                  <div style={{ fontSize: 11, color: link_type === 'audit' ? 'var(--accent-cyan)' : 'var(--text-secondary)', fontWeight: 600 }}>Audit Link</div>
                  <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 2 }}>Requires creator approval</div>
                </div>
              </div>
            </div>
            <div style={{ marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 8 }}>Role</div>
              <div style={{ display: 'flex', gap: 8 }}>
                {['member', 'admin', 'readonly'].map((r) => (
                  <div
                    key={r}
                    style={{ flex: 1, padding: '8px 12px', border: `1px solid ${role === r ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, textAlign: 'center', fontSize: 10, color: role === r ? 'var(--accent-cyan)' : 'var(--text-dim)', background: role === r ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                    onClick={() => set_role(r)}
                  >
                    {r}
                  </div>
                ))}
              </div>
            </div>
            <div style={{ display: 'flex', gap: 12, marginBottom: 20 }}>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Max Uses</div>
                <input
                  type="number"
                  value={max_uses}
                  onChange={(e) => set_max_uses(parseInt(e.target.value, 10) || 0)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Max Members</div>
                <input
                  type="number"
                  value={max_members}
                  placeholder="no limit"
                  onChange={(e) => set_max_members(e.target.value)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Expires (h)</div>
                <input
                  type="number"
                  value={expires_hours}
                  onChange={(e) => set_expires_hours(parseInt(e.target.value, 10) || 1)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
            </div>
            <div
              style={{ padding: 10, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderRadius: 4, textAlign: 'center', fontSize: 11, fontWeight: 700, letterSpacing: 1, cursor: creating ? 'not-allowed' : 'pointer', opacity: creating ? 0.6 : 1 }}
              onClick={creating ? undefined : handle_create}
            >
              {creating ? 'GENERATING...' : 'GENERATE LINK'}
            </div>
          </div>
        </div>
      )}
    </Modal>
  )
}
