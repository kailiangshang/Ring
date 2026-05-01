import { useChatStore } from '../../stores/chat-store'
import { MessageItem } from './MessageItem'
import { ScrollContainer } from '../common/ScrollContainer'

export function MessageList() {
  const messages = useChatStore((s) => s.messages)
  const sending = useChatStore((s) => s.sending)

  return (
    <ScrollContainer autoScroll={sending || undefined}>
      {messages.length === 0 && (
        <div style={{ padding: '48px 16px', textAlign: 'center', color: 'var(--text-dim)', fontSize: 12 }}>
          No messages yet. Start a conversation.
        </div>
      )}
      {messages.map((msg) => (
        <MessageItem key={msg.id} message={msg} />
      ))}
      {/* Bottom padding so last message has breathing room and expand button is easy to click */}
      <div style={{ height: 80 }} />
    </ScrollContainer>
  )
}
