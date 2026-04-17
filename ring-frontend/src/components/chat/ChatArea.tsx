import { MessageList } from './MessageList'
import { InputArea } from './InputArea'

export function ChatArea() {
  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <MessageList />
      <InputArea />
    </div>
  )
}
