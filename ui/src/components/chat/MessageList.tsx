import { useChatStore } from '../../stores/chat-store'
import { MessageItem } from './MessageItem'
import { ScrollContainer } from '../common/ScrollContainer'

export function MessageList() {
  const messages = useChatStore((s) => s.messages)

  return (
    <ScrollContainer>
      {messages.map((msg) => (
        <MessageItem key={msg.id} message={msg} />
      ))}
    </ScrollContainer>
  )
}
