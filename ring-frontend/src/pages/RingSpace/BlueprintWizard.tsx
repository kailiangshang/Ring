import { useEffect, useState, useRef } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useBlueprintStore } from '../../stores/blueprintStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { Tabs } from '../../components/ui/Tabs'
import { Button } from '../../components/ui/Button'
import { EmptyState } from '../../components/ui/EmptyState'
import { GraphPreviewCard } from '../../components/blueprint/GraphPreviewCard'
import type { BlueprintTemplate, GraphDef } from '../../types'
import './BlueprintWizard.css'

type TabMode = 'templates' | 'custom'

export function BlueprintWizard() {
  const { ringId } = useParams<{ ringId: string }>()
  const navigate = useNavigate()
  const [tab, set_tab] = useState<TabMode>('templates')
  const [templates, set_templates] = useState<BlueprintTemplate[]>([])
  const [loading, set_loading] = useState(false)
  const [confirming, set_confirming] = useState(false)
  const [error, set_error] = useState<string | null>(null)
  const messages_end_ref = useRef<HTMLDivElement>(null)

  const messages = useBlueprintStore((s) => s.messages)
  const is_streaming = useBlueprintStore((s) => s.is_streaming)
  const preview_graphs = useBlueprintStore((s) => s.preview_graphs)
  const send_message = useBlueprintStore((s) => s.send_message)
  const confirm = useBlueprintStore((s) => s.confirm)
  const store_error = useBlueprintStore((s) => s.error)

  useEffect(() => {
    if (!ringId || tab !== 'templates') return
    set_loading(true)
    api.list_blueprint_templates(ringId)
      .then(set_templates)
      .catch(() => {})
      .finally(() => set_loading(false))
  }, [ringId, tab])

  useEffect(() => {
    messages_end_ref.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, is_streaming])

  const handle_template_click = async (t: BlueprintTemplate) => {
    if (!ringId) return
    try {
      const parsed_graphs: GraphDef[] = JSON.parse(t.graphs)
      const res = await api.blueprint_preview(ringId, parsed_graphs)
      useBlueprintStore.setState({ preview_graphs: res.graphs })
    } catch (e) {
      set_error((e as Error).message)
    }
  }

  const handle_customize_template = (t: BlueprintTemplate) => {
    if (!ringId) return
    const parsed_graphs: GraphDef[] = JSON.parse(t.graphs)
    api.blueprint_preview(ringId, parsed_graphs).then((res) => {
      useBlueprintStore.setState({ preview_graphs: res.graphs })
      set_tab('custom')
      const initial_msg = `我想基于「${t.name}」模板进行自定义调整。`
      send_message(ringId, initial_msg)
    }).catch(() => {})
  }

  const handle_custom_send = (content: string) => {
    if (!ringId) return
    send_message(ringId, content)
  }

  const handle_confirm = async () => {
    if (!ringId) return
    set_confirming(true)
    try {
      await confirm(ringId)
      navigate(`/ring/${ringId}`)
    } catch {
      // store sets error
    } finally {
      set_confirming(false)
    }
  }

  const handle_edit_graph = () => {
    set_tab('custom')
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

      <div className="blueprint-content">
        {tab === 'templates' && (
          <div className="blueprint-templates">
            {loading && <div className="blueprint-loading">加载模板中...</div>}
            {!loading && templates.length === 0 && (
              <EmptyState
                icon="📐"
                title="没有可用模板"
                description="系统模板尚未配置，请使用「自定义」创建蓝图。"
              />
            )}
            {templates.map((t) => (
              <div
                key={t.id}
                className="blueprint-template-card"
              >
                <div className="blueprint-template-card-main" onClick={() => handle_template_click(t)}>
                  <h3>{t.name}</h3>
                  <p>{t.description}</p>
                </div>
                <div className="blueprint-template-card-actions">
                  <button
                    className="blueprint-template-use"
                    onClick={() => handle_template_click(t)}
                  >
                    使用
                  </button>
                  <button
                    className="blueprint-template-customize"
                    onClick={() => handle_customize_template(t)}
                  >
                    自定义
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}

        {tab === 'custom' && (
          <div className="blueprint-custom">
            {messages.length === 0 && !is_streaming && (
              <div className="blueprint-custom-welcome">
                <div className="blueprint-welcome-icon">🏗️</div>
                <div className="blueprint-welcome-title">从零开始构建蓝图</div>
                <div className="blueprint-welcome-desc">
                  描述你的 Ring 用途，AI 会引导你设计知识图谱结构。
                </div>
                <div className="blueprint-welcome-suggestions">
                  {['这是一个产品竞品分析 Ring', '用于团队学习笔记管理', '项目文档和决策追踪'].map((s) => (
                    <button
                      key={s}
                      className="blueprint-suggestion-chip"
                      onClick={() => handle_custom_send(s)}
                    >
                      {s}
                    </button>
                  ))}
                </div>
              </div>
            )}
            <div className="blueprint-custom-messages">
              {messages.map((msg) => (
                <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
              ))}
              {is_streaming && <div className="blueprint-custom-typing">AI 正在输入...</div>}
              <div ref={messages_end_ref} />
            </div>
            <div className="blueprint-custom-input">
              <ChatInput on_send={handle_custom_send} disabled={is_streaming} />
            </div>
          </div>
        )}
      </div>

      {preview_graphs && preview_graphs.length > 0 && (
        <div className="blueprint-preview">
          <div className="blueprint-preview-header">
            <h3>蓝图预览</h3>
            <span className="blueprint-graph-count">{preview_graphs.length} 个图谱</span>
          </div>
          <div className="blueprint-preview-cards">
            {preview_graphs.map((g, i) => (
              <GraphPreviewCard key={i} graph={g} on_edit={handle_edit_graph} />
            ))}
          </div>
          {preview_graphs.length > 3 && (
            <div className="blueprint-preview-warning">
              建议不超过 3 个图谱，过多的图谱会增加维护复杂度。
            </div>
          )}
          <Button
            onClick={handle_confirm}
            disabled={confirming}
            className="blueprint-confirm-btn"
          >
            {confirming ? '正在确认...' : '确认蓝图'}
          </Button>
        </div>
      )}

      {(error || store_error) && <p className="setup-error" role="alert">{error || store_error}</p>}
    </div>
  )
}
