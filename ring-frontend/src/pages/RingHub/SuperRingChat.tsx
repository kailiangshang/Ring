import { useState } from 'react'
import * as api from '../../api/client'
import { parseSseStream } from '../../components/chat/SseParser'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import type { Message, SseEvent } from '../../types'

export function SuperRingChat() {
  const [messages, set_messages] = useState<Message[]>([])
  const [is_streaming, set_streaming] = useState(false)
  const [error, set_error] = useState<string | null>(null)

  const handle_send = async (content: string) => {
    const user_msg: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: '',
      role: 'user',
      content,
      sender_id: '',
      created_at: new Date().toISOString(),
    }

    set_messages((prev) => [...prev, user_msg])
    set_streaming(true)
    set_error(null)

    try {
      const res = await api.super_ring_chat(content)
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''

      for await (const event of parseSseStream(reader) as AsyncGenerator<SseEvent>) {
        if (event.type === 'text' && event.content) {
          assistant_content += event.content
          set_messages((prev) => {
            const filtered = prev.filter((m) => m.id !== 'stream-super')
            return [
              ...filtered,
              {
                id: 'stream-super',
                conversation_id: '',
                role: 'assistant',
                content: assistant_content,
                sender_id: '',
                created_at: new Date().toISOString(),
              },
            ]
          })
        } else if (event.type === 'error') {
          throw new Error(event.message || 'stream error')
        }
      }
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_streaming(false)
    }
  }

  return (
    <div>
      <h2>Super Ring Chat</h2>
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
          <ChatInput on_send={handle_send} disabled={is_streaming} />
        </div>
      </div>
    </div>
  )
}
