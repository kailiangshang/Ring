import type { ChatMessage } from '../../types/chat'

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  group_ring: 'var(--accent-cyan)',
  super_ring: 'var(--accent-cyan)',
  session_ring: 'var(--accent-teal)',
  self: 'var(--accent-amber)',
  system: 'var(--accent-green)',
}

interface MessageItemProps {
  message: ChatMessage
}

export function MessageItem({ message }: MessageItemProps) {
  const labelColor = ROLE_COLORS[message.role] ?? 'var(--text-muted)'
  const label = message.role === 'user' ? 'YOU' : message.sender_name.toUpperCase()

  return (
    <div style={{ padding: '8px 16px', borderBottom: '1px solid var(--border)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <span style={{ fontSize: 10, fontWeight: 700, color: labelColor, letterSpacing: '0.1em' }}>
          {label}
        </span>
        <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
          {new Date(message.created_at).toLocaleTimeString()}
        </span>
      </div>
      <div style={{ color: 'var(--text-primary)', whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>
        {message.content}
      </div>
    </div>
  )
}
