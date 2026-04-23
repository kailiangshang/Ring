import { useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useSelfChatStore } from '../../stores/self-chat-store'

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  self: 'var(--accent-amber)',
}

/* eslint-disable @typescript-eslint/no-explicit-any */
const md: Record<string, any> = {
  p(props: any) { return <p style={{ margin: '0 0 4px' }}>{props.children}</p> },
  code(props: any) {
    if (props.className?.startsWith('language-')) {
      return <pre style={{ background: 'var(--bg-base)', border: '1px solid var(--border)', borderRadius: 3, padding: '4px 8px', fontSize: 11, overflow: 'auto', margin: '4px 0' }}><code>{props.children}</code></pre>
    }
    return <code style={{ background: 'var(--bg-base)', padding: '0 3px', borderRadius: 2, fontSize: 11, color: 'var(--accent-teal)' }}>{props.children}</code>
  },
  a(props: any) { return <a href={props.href} style={{ color: 'var(--accent-teal)' }} target="_blank" rel="noreferrer">{props.children}</a> },
  ul(props: any) { return <ul style={{ margin: '2px 0', paddingLeft: 16 }}>{props.children}</ul> },
  ol(props: any) { return <ol style={{ margin: '2px 0', paddingLeft: 16 }}>{props.children}</ol> },
  strong(props: any) { return <strong style={{ fontWeight: 700 }}>{props.children}</strong> },
}
/* eslint-enable @typescript-eslint/no-explicit-any */

export function SelfChat() {
  const { messages, input, setInput, send, sending, streaming_message_id, loadHistory } = useSelfChatStore()
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => { loadHistory() }, [loadHistory])

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [messages])

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
      <div ref={scrollRef} style={{ flex: 1, overflow: 'auto', padding: '4px 8px' }}>
        {messages.map((msg) => (
          <div key={msg.id} style={{ marginBottom: 6 }}>
            <span style={{ fontSize: 9, fontWeight: 700, color: ROLE_COLORS[msg.role] ?? 'var(--text-muted)', letterSpacing: '0.08em' }}>
              {msg.role === 'user' ? 'YOU' : 'SELF'}
            </span>
            <div style={{ color: 'var(--text-primary)', fontSize: 12, lineHeight: 1.5 }}>
              <ReactMarkdown remarkPlugins={[remarkGfm]} components={md}>{msg.content}</ReactMarkdown>
              {msg.id === streaming_message_id && (
                <span style={{ display: 'inline-block', width: 5, height: 12, background: 'var(--accent-amber)', marginLeft: 1, verticalAlign: 'middle', animation: 'blink 1s step-end infinite' }} />
              )}
            </div>
          </div>
        ))}
        {messages.length === 0 && (
          <div style={{ color: 'var(--text-dim)', fontSize: 11, textAlign: 'center', paddingTop: 30 }}>
            Chat with your personal AI
          </div>
        )}
      </div>
      <div style={{ display: 'flex', gap: 6, padding: '6px 8px', borderTop: '1px solid var(--border)' }}>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              send()
            }
          }}
          placeholder="Chat with Self..."
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '5px 8px',
            color: 'var(--text-primary)',
            fontSize: 12,
            fontFamily: 'inherit',
            outline: 'none',
          }}
        />
        <button
          onClick={send}
          disabled={sending}
          style={{
            background: 'var(--accent-amber)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            padding: '5px 10px',
            fontSize: 10,
            fontWeight: 700,
            cursor: 'pointer',
            opacity: sending ? 0.5 : 1,
          }}
        >
          {sending ? '...' : '↑'}
        </button>
      </div>
    </div>
  )
}
