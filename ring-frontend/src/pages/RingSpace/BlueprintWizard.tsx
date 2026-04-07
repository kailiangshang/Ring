import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useBlueprintStore } from '../../stores/blueprintStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import type { BlueprintTemplate, GraphDef } from '../../types'

type TabMode = 'templates' | 'custom'

export function BlueprintWizard() {
  const { ringId } = useParams<{ ringId: string }>()
  const [tab, set_tab] = useState<TabMode>('templates')
  const [templates, set_templates] = useState<BlueprintTemplate[]>([])
  const [error, set_error] = useState<string | null>(null)

  const messages = useBlueprintStore((s) => s.messages)
  const is_streaming = useBlueprintStore((s) => s.is_streaming)
  const preview_graphs = useBlueprintStore((s) => s.preview_graphs)
  const send_message = useBlueprintStore((s) => s.send_message)
  const confirm = useBlueprintStore((s) => s.confirm)
  const store_error = useBlueprintStore((s) => s.error)

  useEffect(() => {
    if (!ringId || tab !== 'templates') return
    api.list_blueprint_templates(ringId).then(set_templates).catch(() => {})
  }, [ringId, tab])

  const handle_template_click = async (t: BlueprintTemplate) => {
    if (!ringId) return
    try {
      const res = await api.blueprint_preview(ringId, t.graphs)
      useBlueprintStore.setState({ preview_graphs: res.graphs })
    } catch (e) {
      set_error((e as Error).message)
    }
  }

  const handle_custom_send = (content: string) => {
    if (!ringId) return
    send_message(ringId, content)
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
        <div style={{ marginTop: 16, border: '1px solid #aa3bff', padding: 12, background: 'var(--bg, #fff)', color: 'var(--text-h, #000)', borderRadius: 8 }}>
          <h3>Blueprint Preview</h3>
          {preview_graphs.map((g, i) => (
            <div key={i}>
              <strong>{g.name}</strong> ({g.graph_type}) —{' '}
              {g.categories.join(', ')}
            </div>
          ))}
          <button onClick={() => ringId && confirm(ringId)} style={{ marginTop: 8 }}>
            Confirm Blueprint
          </button>
        </div>
      )}

      {(error || store_error) && <p role="alert">{error || store_error}</p>}
    </div>
  )
}
