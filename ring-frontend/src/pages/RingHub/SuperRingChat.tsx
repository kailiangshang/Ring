import { useSuperRingStore } from '../../stores/superRingStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import './RingHub.css'

export function SuperRingChat() {
  const messages = useSuperRingStore((s) => s.messages)
  const is_streaming = useSuperRingStore((s) => s.is_streaming)
  const error = useSuperRingStore((s) => s.error)
  const send_message = useSuperRingStore((s) => s.send_message)

  return (
    <div>
      <div className="super-ring-header">
        <h2 className="super-ring-title">Super Ring</h2>
        <p className="super-ring-subtitle">全局助手</p>
      </div>
      <div className="super-ring-chat">
        <div className="super-ring-messages">
          {messages.map((msg) => (
            <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
          ))}
          {is_streaming && <div className="super-ring-typing">AI is typing...</div>}
        </div>
        {error && <p className="setup-error" role="alert">{error}</p>}
        <div className="super-ring-input-area">
          <ChatInput on_send={send_message} disabled={is_streaming} />
        </div>
      </div>
    </div>
  )
}
