import { useState, useEffect, useRef } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useAuthStore } from '../../stores/auth-store'
import { verifyJoinToken, localJoin, applyJoin, checkApplyStatus } from '../../services/api'
import type { JoinInfo } from '../../types/invite'

interface StepJoinProps {
  initial_token?: string
  initial_creator_ip?: string
}

export function StepJoin({ initial_token, initial_creator_ip }: StepJoinProps) {
  const [invite_link, set_invite_link] = useState(initial_token ? `token=${initial_token}` : '')
  const [creator_ip, set_creator_ip] = useState(initial_creator_ip || '')
  const [display_name, set_display_name] = useState('')
  const [join_info, set_join_info] = useState<JoinInfo | null>(null)
  const [message, set_message] = useState('')
  const [error, set_error] = useState<string | null>(null)
  const [loading, set_loading] = useState(false)
  const [status, set_status] = useState<'idle' | 'verified' | 'joining' | 'polling' | 'done'>('idle')
  const pollCleanup = useRef<(() => void) | null>(null)

  const setSetup = useAppStore((s) => s.setSetup)
  const setAuth = useAuthStore((s) => s.setAuth)

  useEffect(() => {
    if (initial_token) {
      handle_verify(initial_token)
    }
    return () => { pollCleanup.current?.() }
  }, [])

  const parse_token = (input: string): { token: string; ip?: string } => {
    const trimmed = input.trim()
    try {
      const url = new URL(trimmed)
      const token = url.searchParams.get('token') || ''
      const ip = url.searchParams.get('creator_ip') || url.hostname
      return { token, ip: ip || undefined }
    } catch {
      if (trimmed.includes('=')) {
        const params = new URLSearchParams(trimmed)
        return { token: params.get('token') || '', ip: params.get('creator_ip') || undefined }
      }
      return { token: trimmed }
    }
  }

  const handle_verify = async (token_input?: string) => {
    const input = token_input || invite_link
    const { token, ip } = parse_token(input)
    if (!token) { set_error('No token found'); return }
    if (ip) set_creator_ip(ip)
    set_loading(true)
    set_error(null)
    try {
      const info = await verifyJoinToken(token)
      if (info.valid) {
        set_join_info(info)
        set_status('verified')
      } else {
        set_error(info.reason || 'Invalid invite link')
      }
    } catch {
      set_error('Failed to verify invite link')
    } finally {
      set_loading(false)
    }
  }

  const handle_join = async () => {
    if (!display_name.trim()) { set_error('Display name is required'); return }
    const { token } = parse_token(invite_link)
    if (!token) return
    set_loading(true)
    set_error(null)
    set_status('joining')
    try {
      if (join_info?.token_type === 'audit') {
        const res = await applyJoin(token, display_name.trim(), message || undefined)
        set_status('polling')
        pollCleanup.current = poll_status(res.request_id)
      } else if (creator_ip) {
        const res = await localJoin(token, creator_ip)
        setAuth(res.ring_id, display_name.trim(), null)
        set_status('done')
        setSetup(true)
      } else {
        set_error('Creator IP is required for open join')
        set_loading(false)
      }
    } catch (e: unknown) {
      set_error(e instanceof Error ? e.message : 'Join failed')
      set_loading(false)
    }
  }

  const poll_status = (request_id: string) => {
    const interval = setInterval(async () => {
      try {
        const res = await checkApplyStatus(request_id)
        if (res.status === 'approved') {
          clearInterval(interval)
          set_status('done')
          setSetup(true)
        } else if (res.status === 'rejected') {
          clearInterval(interval)
          set_error(res.review_note ? `Rejected: ${res.review_note}` : 'Application rejected')
          set_loading(false)
        }
      } catch {
      }
    }, 3000)
    return () => clearInterval(interval)
  }

  const input_style = { width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontFamily: 'inherit' as const, fontSize: 12, outline: 'none' as const }

  return (
    <div style={{ maxWidth: 480, width: '100%', padding: '40px 20px', textAlign: 'center' }}>
      <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 6 }}>Join a Ring</h1>
      <p style={{ color: 'var(--text-secondary)', marginBottom: 24, fontSize: 12 }}>Paste the invite link shared by the Ring creator.</p>

      {status === 'idle' && (
        <>
          <div style={{ textAlign: 'left', marginBottom: 16 }}>
            <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Invite Link / Code</label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input value={invite_link} onChange={(e) => set_invite_link(e.target.value)} placeholder="http://192.168.x.x:7420/ring/join?token=xxx" style={{ ...input_style, flex: 1 }} />
              <button onClick={() => handle_verify()} disabled={loading} style={{ padding: '8px 16px', background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, fontSize: 10, fontWeight: 700, cursor: 'pointer', letterSpacing: 1 }}>VERIFY</button>
            </div>
          </div>
          {error && <p style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 12 }}>{error}</p>}
        </>
      )}

      {status === 'verified' && join_info && (
        <div style={{ textAlign: 'left' }}>
          <div style={{ background: 'var(--bg-active)', border: '1px solid var(--accent-cyan)', borderRadius: 4, padding: 12, marginBottom: 16 }}>
            <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--accent-ice)', marginBottom: 4 }}>{join_info.ring_name}</div>
            <div style={{ fontSize: 10, color: 'var(--text-secondary)' }}>Members: {join_info.member_count} · Role: {join_info.role} · Type: {join_info.token_type}</div>
          </div>
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Display Name</label>
            <input value={display_name} onChange={(e) => set_display_name(e.target.value)} placeholder="Your name" style={input_style} />
          </div>
          {join_info.token_type === 'audit' && (
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Message (optional)</label>
              <input value={message} onChange={(e) => set_message(e.target.value)} placeholder="Why do you want to join?" style={input_style} />
            </div>
          )}
          {error && <p style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 12 }}>{error}</p>}
          <button onClick={handle_join} disabled={loading} style={{ width: '100%', padding: 10, background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, fontSize: 11, fontWeight: 700, cursor: loading ? 'not-allowed' : 'pointer', letterSpacing: 1, opacity: loading ? 0.6 : 1 }}>
            {loading ? 'JOINING...' : `JOIN "${join_info.ring_name}"`}
          </button>
        </div>
      )}

      {status === 'polling' && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>Application submitted. Waiting for approval...</p>
          <p style={{ color: 'var(--text-dim)', fontSize: 10, marginTop: 8 }}>This page will auto-update when approved.</p>
        </div>
      )}

      {status === 'done' && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <div style={{ fontSize: 32, marginBottom: 12 }}>🎉</div>
          <p style={{ color: 'var(--accent-green)', fontSize: 14, fontWeight: 600 }}>Successfully joined!</p>
        </div>
      )}
    </div>
  )
}
