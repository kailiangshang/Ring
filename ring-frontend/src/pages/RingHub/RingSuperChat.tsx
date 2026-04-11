import { useRingSuperStore } from '../../stores/ringSuperStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import './RingHub.css'

export function RingSuperChat() {
  const messages = useRingSuperStore((s) => s.messages)
  const is_streaming = useRingSuperStore((s) => s.is_streaming)
  const error = useRingSuperStore((s) => s.error)
  const send_message = useRingSuperStore((s) => s.send_message)

  return (
    <div>
      <div className="ring-super-header">
        <h2 className="ring-super-title">Ring Super</h2>
        <p className="ring-super-subtitle">全局助手</p>
      </div>
      <div className="ring-super-chat">
        <div className="ring-super-messages">
          {messages.map((msg) => (
            <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
          ))}
          {is_streaming && <div className="ring-super-typing">AI is typing...</div>}
        </div>
        {error && <p className="setup-error" role="alert">{error}</p>}
        <div className="ring-super-input-area">
          <ChatInput on_send={send_message} disabled={is_streaming} />
        </div>
      </div>
    </div>
  )
}
