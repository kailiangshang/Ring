import { useState } from 'react'
import type { ExportType, ExportTypeInfo, ExportFormat } from '../../types'
import { ExportTypeList } from './ExportTypeList'
import { ExportConfig } from './ExportConfig'
import './ExportPanel.css'

export const EXPORT_TYPES: ExportTypeInfo[] = [
  { type: 'graph_image', label: '图谱图片', description: '导出知识图谱为图片', icon: '🖼️', formats: ['png', 'svg', 'pdf'], group: 'graph' },
  { type: 'graph_json', label: 'graph.json', description: '导出图谱原始 JSON 数据', icon: '📊', formats: ['json'], group: 'graph' },
  { type: 'markdown', label: '单篇 Markdown', description: '导出节点对应的 Markdown 文件', icon: '📝', formats: ['markdown'], group: 'content' },
  { type: 'conversation', label: '对话记录', description: '导出完整对话历史', icon: '💬', formats: ['markdown', 'pdf'], group: 'content' },
  { type: 'session', label: 'Session 讨论记录', description: '导出 Session 完整讨论', icon: '👥', formats: ['markdown', 'pdf'], group: 'content' },
  { type: 'report', label: 'AI 结构化报告', description: 'AI 基于节点内容生成报告', icon: '🤖', formats: ['markdown', 'pdf'], group: 'ai' },
  { type: 'backup', label: '整 Ring 备份', description: '导出 Ring 全部数据 (.tar.gz)', icon: '📦', formats: ['tar.gz'], group: 'backup' },
]

interface ExportPanelProps {
  ring_id: string
  on_close: () => void
}

export function ExportPanel({ ring_id, on_close }: ExportPanelProps) {
  const [selected_type, set_selected_type] = useState<ExportType | null>(null)
  const [exporting, set_exporting] = useState(false)
  const [error, set_error] = useState<string | null>(null)
  const [success, set_success] = useState(false)

  const handle_export = async (config: Record<string, unknown>) => {
    set_exporting(true)
    set_error(null)
    set_success(false)
    try {
      const { default: api } = await import('../../api/client')
      const type = selected_type!
      switch (type) {
        case 'graph_image':
          await api.export_graph_image(ring_id, {
            graph_id: config.graph_id as string,
            format: (config.format as 'png' | 'svg' | 'pdf') || 'svg',
          })
          break
        case 'graph_json':
          await api.export_graph_json(ring_id, config.graph_id as string)
          break
        case 'markdown':
          await api.export_markdown(ring_id, config.node_id as string)
          break
        case 'conversation':
          await api.export_conversation(ring_id, {
            conversation_id: config.conversation_id as string,
            format: (config.format as 'markdown' | 'pdf') || 'markdown',
            include_ai_responses: config.include_ai_responses !== false,
          })
          break
        case 'session':
          await api.export_session(ring_id, {
            session_id: config.session_id as string,
            format: (config.format as 'markdown' | 'pdf') || 'markdown',
          })
          break
        case 'report':
          await api.export_report(ring_id, {
            topic: config.topic as string,
            node_ids: config.node_ids as string[],
            graph_id: config.graph_id as string,
            format: (config.format as 'markdown' | 'pdf') || 'markdown',
          })
          break
        case 'backup':
          await api.export_backup(ring_id)
          break
      }
      set_success(true)
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_exporting(false)
    }
  }

  const type_info = EXPORT_TYPES.find((t) => t.type === selected_type)

  return (
    <div className="export-panel-overlay" onClick={on_close}>
      <div className="export-panel" onClick={(e) => e.stopPropagation()}>
        <div className="export-panel-header">
          <h3>导出中心</h3>
          <button className="export-panel-close" onClick={on_close}>✕</button>
        </div>
        <div className="export-panel-body">
          <ExportTypeList
            selected={selected_type}
            on_select={set_selected_type}
          />
          <div className="export-panel-config">
            {selected_type && type_info ? (
              <ExportConfig
                type_info={type_info}
                ring_id={ring_id}
                exporting={exporting}
                error={error}
                success={success}
                on_export={handle_export}
              />
            ) : (
              <div className="export-config-empty">选择左侧导出类型</div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
