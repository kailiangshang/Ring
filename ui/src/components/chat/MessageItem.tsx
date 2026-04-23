import type { ChatMessage } from '../../types/chat'
import { useChatStore } from '../../stores/chat-store'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  group_ring: 'var(--accent-cyan)',
  super_ring: 'var(--accent-cyan)',
  session_ring: 'var(--accent-teal)',
  self: 'var(--accent-amber)',
  system: 'var(--accent-green)',
}

/* eslint-disable @typescript-eslint/no-explicit-any */
const mdComponents: Record<string, any> = {
  p(props: any) { return <p style={{ margin: '0 0 8px' }}>{props.children}</p> },
  h1(props: any) { return <h1 style={{ fontSize: 18, fontWeight: 700, color: 'var(--accent-ice)', margin: '12px 0 6px' }}>{props.children}</h1> },
  h2(props: any) { return <h2 style={{ fontSize: 15, fontWeight: 700, color: 'var(--accent-ice)', margin: '10px 0 4px' }}>{props.children}</h2> },
  h3(props: any) { return <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-ice)', margin: '8px 0 4px' }}>{props.children}</h3> },
  pre(props: any) { return <div style={{ margin: '6px 0' }}>{props.children}</div> },
  ul(props: any) { return <ul style={{ margin: '4px 0', paddingLeft: 20 }}>{props.children}</ul> },
  ol(props: any) { return <ol style={{ margin: '4px 0', paddingLeft: 20 }}>{props.children}</ol> },
  li(props: any) { return <li style={{ marginBottom: 2 }}>{props.children}</li> },
  blockquote(props: any) { return <blockquote style={{ borderLeft: '3px solid var(--accent-cyan)', margin: '6px 0', paddingLeft: 10, color: 'var(--text-secondary)' }}>{props.children}</blockquote> },
  strong(props: any) { return <strong style={{ fontWeight: 700, color: 'var(--text-primary)' }}>{props.children}</strong> },
  hr() { return <hr style={{ border: 'none', borderTop: '1px solid var(--border)', margin: '8px 0' }} /> },
  code(props: any) {
    const { className, children } = props
    if (className?.startsWith('language-')) {
      return (
        <pre style={{
          background: 'var(--bg-base)',
          border: '1px solid var(--border)',
          borderRadius: 3,
          padding: '8px 12px',
          fontSize: 12,
          overflow: 'auto',
          margin: '6px 0',
        }}>
          <code>{children}</code>
        </pre>
      )
    }
    return (
      <code style={{
        background: 'var(--bg-base)',
        padding: '1px 4px',
        borderRadius: 3,
        fontSize: 12,
        color: 'var(--accent-teal)',
      }}>{children}</code>
    )
  },
  a(props: any) {
    return <a href={props.href} style={{ color: 'var(--accent-teal)', textDecoration: 'underline' }} target="_blank" rel="noreferrer">{props.children}</a>
  },
  table(props: any) { return <table style={{ borderCollapse: 'collapse', margin: '6px 0', fontSize: 11, width: '100%' }}>{props.children}</table> },
  th(props: any) { return <th style={{ border: '1px solid var(--border)', padding: '4px 8px', textAlign: 'left', fontWeight: 700, color: 'var(--accent-ice)' }}>{props.children}</th> },
  td(props: any) { return <td style={{ border: '1px solid var(--border)', padding: '4px 8px' }}>{props.children}</td> },
}
/* eslint-enable @typescript-eslint/no-explicit-any */

interface MessageItemProps {
  message: ChatMessage
}

export function MessageItem({ message }: MessageItemProps) {
  const streaming_message_id = useChatStore((s) => s.streaming_message_id)
  const isStreaming = message.id === streaming_message_id
  const labelColor = ROLE_COLORS[message.role] ?? 'var(--text-muted)'
  const label = message.role === 'user' ? 'YOU' : message.sender_name.toUpperCase()
  const isUser = message.role === 'user'

  return (
    <div style={{
      padding: '8px 16px',
      borderBottom: '1px solid var(--border)',
      display: 'flex',
      justifyContent: isUser ? 'flex-end' : 'flex-start',
    }}>
      <div style={{
        maxWidth: '85%',
        background: isUser ? 'var(--bg-active)' : 'transparent',
        borderRadius: isUser ? '6px 6px 2px 6px' : 0,
        padding: isUser ? '8px 12px' : 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, justifyContent: isUser ? 'flex-end' : 'flex-start' }}>
          <span style={{ fontSize: 10, fontWeight: 700, color: labelColor, letterSpacing: '0.1em' }}>
            {label}
          </span>
          <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
            {new Date(message.created_at).toLocaleTimeString()}
          </span>
        </div>
        <div style={{ color: 'var(--text-primary)', lineHeight: 1.6, fontSize: 13 }}>
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={mdComponents}>
            {message.content}
          </ReactMarkdown>
          {isStreaming && (
            <span style={{
              display: 'inline-block',
              width: 6,
              height: 14,
              background: 'var(--accent-cyan)',
              marginLeft: 2,
              verticalAlign: 'middle',
              animation: 'blink 1s step-end infinite',
            }} />
          )}
        </div>
        {message.token_usage && message.role !== 'user' && (
          <div style={{ marginTop: 4, fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8 }}>
            {message.token_usage.prompt_tokens !== undefined && (
              <span>prompt: {message.token_usage.prompt_tokens}</span>
            )}
            {message.token_usage.completion_tokens !== undefined && (
              <span>completion: {message.token_usage.completion_tokens}</span>
            )}
            {message.token_usage.total_tokens !== undefined && (
              <span>total: {message.token_usage.total_tokens}</span>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
