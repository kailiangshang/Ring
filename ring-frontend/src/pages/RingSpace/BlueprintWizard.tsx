import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import * as api from '../../api/client'
import { parseSseStream } from '../../components/chat/SseParser'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import type {
  BlueprintTemplate,
  Message,
  SseEvent,
  GraphDef,
} from '../../types'

type TabMode = 'templates' | 'custom'

export function BlueprintWizard() {
  const { ringId } = useParams<{ ringId: string }>()
  const [tab, set_tab] = useState<TabMode>('templates')
  const [templates, set_templates] = useState<BlueprintTemplate[]>([])
  const [messages, set_messages] = useState<Message[]>([])
  const [is_streaming, set_streaming] = useState(false)
  const [preview_graphs, set_preview_graphs] = useState<GraphDef[] | null>(null)
  const [error, set_error] = useState<string | null>(null)

  useEffect(() => {
    if (!ringId || tab !== 'templates') return
    api.list_blueprint_templates(ringId).then(set_templates).catch(() => {})
  }, [ringId, tab])

  const handle_template_click = async (t: BlueprintTemplate) => {
    if (!ringId) return
    try {
      const res = await api.blueprint_preview(ringId, t.graphs)
      set_preview_graphs(res.graphs)
    } catch (e) {
      set_error((e as Error).message)
    }
  }

  const handle_confirm = async () => {
    if (!ringId || !preview_graphs) return
    try {
      await api.blueprint_confirm(ringId, preview_graphs)
      set_preview_graphs(null)
    } catch (e) {
      set_error((e as Error).message)
    }
  }

  const handle_custom_send = async (content: string) => {
    if (!ringId) return

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
      const res = await api.blueprint_chat(ringId, content)
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
            const filtered = prev.filter((m) => m.id !== 'stream-blueprint')
            return [
              ...filtered,
              {
                id: 'stream-blueprint',
                conversation_id: '',
                role: 'assistant',
                content: assistant_content,
                sender_id: '',
                created_at: new Date().toISOString(),
              },
            ]
          })
        } else if (event.type === 'blueprint_proposal' && event.graphs) {
          set_preview_graphs(event.graphs)
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
    <div style={{ padding: 16 }}>
      <h2>Blueprint Wizard</h2>
      <div style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <button
          onClick={() => set_tab('templates')}
          style={{ fontWeight: tab === 'templates' ? 'bold' : 'normal' }}
        >
          Templates
        </button>
        <button
          onClick={() => set_tab('custom')}
          style={{ fontWeight: tab === 'custom' ? 'bold' : 'normal' }}
        >
          Custom
        </button>
      </div>

      {tab === 'templates' && (
        <div>
          {templates.length === 0 && <p>No templates available.</p>}
          {templates.map((t) => (
            <div
              key={t.id}
              onClick={() => handle_template_click(t)}
              style={{
                border: '1px solid #ccc',
                padding: 12,
                marginBottom: 8,
                cursor: 'pointer',
              }}
            >
              <h3>{t.name}</h3>
              <p>{t.description}</p>
            </div>
          ))}
        </div>
      )}

      {tab === 'custom' && (
        <div style={{ display: 'flex', flexDirection: 'column', height: '60vh' }}>
          <div style={{ flex: 1, overflow: 'auto' }}>
            {messages.map((msg) => (
              <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
            ))}
            {is_streaming && (
              <div style={{ color: '#888', marginBottom: 8 }}>AI is typing...</div>
            )}
          </div>
          <ChatInput on_send={handle_custom_send} disabled={is_streaming} />
        </div>
      )}

      {preview_graphs && (
        <div style={{ marginTop: 16, border: '1px solid #aa3bff', padding: 12 }}>
          <h3>Blueprint Preview</h3>
          {preview_graphs.map((g, i) => (
            <div key={i}>
              <strong>{g.name}</strong> ({g.graph_type}) —{' '}
              {g.categories.join(', ')}
            </div>
          ))}
          <button onClick={handle_confirm} style={{ marginTop: 8 }}>
            Confirm Blueprint
          </button>
        </div>
      )}

      {error && <p role="alert">{error}</p>}
    </div>
  )
}
