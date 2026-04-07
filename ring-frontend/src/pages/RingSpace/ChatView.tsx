import { useEffect, useRef, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../stores/chatStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { ToolCallBubble } from '../../components/chat/ToolCallBubble'
import { ToolResultBubble } from '../../components/chat/ToolResultBubble'
import { ArchiveSuggestion } from '../../components/chat/ArchiveSuggestion'
import { Toolbar } from '../../components/toolbar/Toolbar'
import type { ToolStatus } from '../../components/toolbar/Toolbar'

const DEFAULT_TOOLS: ToolStatus[] = [
  { name: 'search', description: 'Search the knowledge graph', active: false },
  { name: 'text_clean', description: 'Clean and normalize text', active: false },
  { name: 'web_scrape', description: 'Extract text from web pages', active: false },
  { name: 'markdown_gen', description: 'Generate markdown documents', active: false },
  { name: 'privacy_filter', description: 'Filter sensitive information', active: false },
]

export function ChatView() {
  const { ringId } = useParams<{ ringId: string }>()
  const {
    messages,
    tool_events,
    is_streaming,
    error,
    current_conversation_id,
    create_conversation,
    load_history,
    send_message,
    reset,
  } = useChatStore()
  const bottom_ref = useRef<HTMLDivElement>(null)
  const [tools, set_tools] = useState<ToolStatus[]>(DEFAULT_TOOLS)

  const handle_toggle = (tool_name: string) => {
    set_tools((prev) =>
      prev.map((t) => (t.name === tool_name ? { ...t, active: !t.active } : t)),
    )
  }

  useEffect(() => {
    if (!ringId) return
    if (current_conversation_id) return

    const init = async () => {
      try {
        const convs = await api.list_conversations(ringId)
        if (convs.length > 0) {
          const last = convs[convs.length - 1]
          await load_history(ringId, last.id)
        } else {
          reset()
          const conv_id = await create_conversation(ringId, 'New Conversation')
          await load_history(ringId, conv_id)
        }
      } catch {
        reset()
        const conv_id = await create_conversation(ringId, 'New Conversation')
        await load_history(ringId, conv_id)
      }
    }

    init()
  }, [ringId])

  useEffect(() => {
    bottom_ref.current?.scrollIntoView?.({ behavior: 'smooth' })
  }, [messages, tool_events])

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
        {tool_events.map((evt) => {
          if (evt.type === 'tool_call') {
            const done = tool_events.some(
              (r) => r.type === 'tool_result' && r.tool_call_id === evt.tool_call_id,
            )
            return (
              <ToolCallBubble
                key={evt.id}
                tool_name={evt.tool_name ?? 'unknown'}
                input={evt.input}
                done={done}
              />
            )
          }
          if (evt.type === 'tool_result') {
            return (
              <ToolResultBubble
                key={evt.id}
                tool_name={evt.tool_name ?? 'unknown'}
                output={evt.output}
                success={evt.success}
              />
            )
          }
          if (evt.type === 'archive_suggestion') {
            return (
              <ArchiveSuggestion
                key={evt.id}
                data={evt.data}
                on_accept={() => {}}
                on_dismiss={() => {}}
              />
            )
          }
          return null
        })}
        {is_streaming && (
          <div style={{ textAlign: 'left', marginBottom: 8, color: '#888' }}>
            AI is typing...
          </div>
        )}
        <div ref={bottom_ref} />
      </div>
      {error && <p role="alert">{error}</p>}
      <div style={{ padding: 16, borderTop: '1px solid #eee' }}>
        <Toolbar tools={tools} on_toggle={handle_toggle} />
        <ChatInput on_send={handle_send} disabled={is_streaming} />
      </div>
    </div>
  )
}
