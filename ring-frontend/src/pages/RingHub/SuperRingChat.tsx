import { useSuperRingStore } from '../../stores/superRingStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'

export function SuperRingChat() {
  const messages = useSuperRingStore((s) => s.messages)
  const is_streaming = useSuperRingStore((s) => s.is_streaming)
  const error = useSuperRingStore((s) => s.error)
  const send_message = useSuperRingStore((s) => s.send_message)

  return (
    <div>
      <h2>Ring Super Chat</h2>
      <div style={{ display: 'flex', flexDirection: 'column', height: '70vh' }}>
        <div style={{ flex: 1, overflow: 'auto', padding: 16 }}>
          {messages.map((msg) => (
            <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
          ))}
          {is_streaming && (
            <div style={{ color: '#888', marginBottom: 8 }}>AI is typing...</div>
          )}
        </div>
        {error && <p role="alert">{error}</p>}
        <div style={{ padding: 16, borderTop: '1px solid #eee' }}>
          <ChatInput on_send={send_message} disabled={is_streaming} />
        </div>
      </div>
    </div>
  )
}
