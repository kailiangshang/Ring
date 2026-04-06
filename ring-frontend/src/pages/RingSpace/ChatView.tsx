import { useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../stores/chatStore'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'

export function ChatView() {
  const { ringId } = useParams<{ ringId: string }>()
  const {
    messages,
    is_streaming,
    error,
    current_conversation_id,
    create_conversation,
    load_history,
    send_message,
    reset,
  } = useChatStore()
  const bottom_ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ringId) return

    const init = async () => {
      reset()
      const conv_id = await create_conversation(ringId, 'New Conversation')
      await load_history(ringId, conv_id)
    }

    init()
  }, [ringId])

  useEffect(() => {
    bottom_ref.current?.scrollIntoView?.({ behavior: 'smooth' })
  }, [messages])

  const handle_send = (content: string) => {
    if (!ringId || !current_conversation_id) return
    send_message(ringId, content)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '80vh' }}>
      <div style={{ flex: 1, overflow: 'auto', padding: 16 }}>
        {messages.map((msg) => (
          <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
        ))}
        {is_streaming && (
          <div style={{ textAlign: 'left', marginBottom: 8, color: '#888' }}>
            AI is typing...
          </div>
        )}
        <div ref={bottom_ref} />
      </div>
      {error && <p role="alert">{error}</p>}
      <div style={{ padding: 16, borderTop: '1px solid #eee' }}>
        <ChatInput on_send={handle_send} disabled={is_streaming} />
      </div>
    </div>
  )
}
