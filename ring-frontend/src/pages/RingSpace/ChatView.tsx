import { useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../stores/chatStore'
import { useGraphStore } from '../../stores/graphStore'
import { useModeStore } from '../../stores/modeStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { ToolCallBubble } from '../../components/chat/ToolCallBubble'
import { ToolResultBubble } from '../../components/chat/ToolResultBubble'
import { ArchiveSuggestion, type ArchiveSuggestionData } from '../../components/chat/ArchiveSuggestion'
import { ArchiveConfirmDialog } from '../../components/archive/ArchiveConfirmDialog'
import { useTools } from '../../components/layout/RingSpaceLayout'
import './ChatView.css'

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
    archive_pending,
    trigger_archive,
    dismiss_suggestion,
    clear_archive_pending,
  } = useChatStore()
  const { graphs, current_graph_id, nodes, load_graphs, select_graph } = useGraphStore()
  const mode = useModeStore((s) => s.mode)
  const bottom_ref = useRef<HTMLDivElement>(null)
  const { active_tool_names } = useTools()

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
    if (!ringId || graphs.length > 0) return
    load_graphs(ringId).then(() => {
      const g = useGraphStore.getState().graphs
      if (g.length > 0) select_graph(ringId, g[0])
    })
  }, [ringId])

  useEffect(() => {
    bottom_ref.current?.scrollIntoView?.({ behavior: 'smooth' })
  }, [messages, tool_events])

  const handle_send = (content: string) => {
    if (!ringId || !current_conversation_id) return
    send_message(ringId, content, active_tool_names.length > 0 ? active_tool_names : undefined)
  }

  const handle_export = () => {
    if (!ringId) return
    const graph_id = current_graph_id || graphs[0]
    if (!graph_id) return
    trigger_archive(ringId, graph_id)
  }

  const handle_suggestion_accept = async (suggestion: ArchiveSuggestionData) => {
    if (!ringId || !current_conversation_id) return
    const graph_id = current_graph_id || graphs[0]
    if (!graph_id) return

    const unarchived = messages.filter((m) => !m.archived).slice(-5)
    const msg_ids = unarchived.length > 0 ? unarchived.map((m) => m.id) : []
    const last_user_msg = [...messages].reverse().find((m) => m.role === 'user')
    const label = suggestion.suggested_title || (last_user_msg?.content || 'Archive').slice(0, 30)

    try {
      const res = await api.archive_content(ringId, {
        message_ids: msg_ids,
        conversation_id: current_conversation_id,
        graph_id,
        label,
        target_node_id: suggestion.target_node_id,
      })
      useChatStore.setState({
        archive_pending: {
          archive_id: res.archive_id,
          suggested_title: suggestion.suggested_title,
          suggested_parent: suggestion.suggested_parent,
          message_ids: msg_ids,
          conversation_id: current_conversation_id,
          graph_id,
          label,
        },
      })
    } catch (e) {
      useChatStore.setState({ error: (e as Error).message })
    }
  }

  const handle_archive_confirm = async (target_node_id: string | undefined) => {
    if (!ringId || !archive_pending) return
    if (archive_pending.archive_id) {
      await api.confirm_archive(ringId, archive_pending.archive_id)
    } else {
      await api.archive_content(ringId, {
        message_ids: archive_pending.message_ids,
        conversation_id: archive_pending.conversation_id,
        graph_id: archive_pending.graph_id,
        label: archive_pending.label,
        target_node_id,
      })
    }
    clear_archive_pending()
  }

  const show_export = mode === 'manual_archive'

  return (
    <div className="chat-view">
      <div className="chat-header">Chat</div>
      <div className="chat-messages">
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
                on_accept={handle_suggestion_accept}
                on_dismiss={() => dismiss_suggestion(evt.id)}
              />
            )
          }
          return null
        })}
        {is_streaming && <div className="chat-typing">AI is typing...</div>}
        <div ref={bottom_ref} />
      </div>
      {error && <p className="chat-error" role="alert">{error}</p>}
      <div className="chat-input-area">
        {show_export && (
          <button className="chat-export-btn" onClick={handle_export} disabled={is_streaming} title="归档">
            📥
          </button>
        )}
        <ChatInput on_send={handle_send} disabled={is_streaming} />
      </div>
      <ArchiveConfirmDialog
        open={archive_pending !== null}
        on_close={clear_archive_pending}
        suggested_title={archive_pending?.suggested_title}
        suggested_parent={archive_pending?.suggested_parent}
        nodes={nodes}
        on_confirm={handle_archive_confirm}
      />
    </div>
  )
}
