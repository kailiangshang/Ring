import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useBlueprintStore } from '../../stores/blueprintStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { Tabs } from '../../components/ui/Tabs'
import { Button } from '../../components/ui/Button'
import { EmptyState } from '../../components/ui/EmptyState'
import type { BlueprintTemplate } from '../../types'
import './BlueprintWizard.css'

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
    <div className="blueprint-wizard">
      <div className="blueprint-header">
        <h2>Blueprint Wizard</h2>
        <Tabs
          tabs={[
            { key: 'templates', label: '模板' },
            { key: 'custom', label: '自定义' },
          ]}
          active_key={tab}
          on_change={(k) => set_tab(k as TabMode)}
        />
      </div>

      {tab === 'templates' && (
        <div className="blueprint-templates">
          {templates.length === 0 && (
            <EmptyState
              icon="📐"
              title="No templates available"
              description="Blueprint templates will appear here when configured."
            />
          )}
          {templates.map((t) => (
            <div
              key={t.id}
              className="blueprint-template-card"
              onClick={() => handle_template_click(t)}
            >
              <h3>{t.name}</h3>
              <p>{t.description}</p>
            </div>
          ))}
        </div>
      )}

      {tab === 'custom' && (
        <div className="blueprint-custom">
          <div className="blueprint-custom-messages">
            {messages.map((msg) => (
              <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
            ))}
            {is_streaming && <div className="blueprint-custom-typing">AI is typing...</div>}
          </div>
          <div className="blueprint-custom-input">
            <ChatInput on_send={handle_custom_send} disabled={is_streaming} />
          </div>
        </div>
      )}

      {preview_graphs && (
        <div className="blueprint-preview">
          <h3>Blueprint Preview</h3>
          {preview_graphs.map((g, i) => (
            <div key={i} className="blueprint-preview-item">
              <strong>{g.name}</strong> ({g.graph_type}) — {g.categories.join(', ')}
            </div>
          ))}
          <Button onClick={() => ringId && confirm(ringId)} className="blueprint-confirm-btn">
            Confirm Blueprint
          </Button>
        </div>
      )}

      {(error || store_error) && <p className="setup-error" role="alert">{error || store_error}</p>}
    </div>
  )
}
