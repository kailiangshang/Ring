import { useEffect, useRef } from 'react'
import { MarkdownRenderer } from '../common/MarkdownRenderer'
import { useSelfChatStore } from '../../stores/self-chat-store'

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  self: 'var(--accent-amber)',
}

export function SelfChat() {
  const { messages, input, setInput, send, sending, streaming_message_id, stopStreaming, loadHistory } = useSelfChatStore()
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
              <MarkdownRenderer content={msg.content} />
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
        <textarea
          className="self-chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              send()
            }
          }}
          placeholder="Chat with Self..."
          rows={1}
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
            resize: 'none',
          }}
        />
        {sending ? (
          <button
            onClick={stopStreaming}
            style={{
              background: 'var(--accent-amber)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '5px 10px',
              fontSize: 10,
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            STOP
          </button>
        ) : (
          <button
            onClick={send}
            style={{
              background: 'var(--accent-amber)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '5px 10px',
              fontSize: 10,
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            {'\u2191'}
          </button>
        )}
      </div>
    </div>
  )
}
