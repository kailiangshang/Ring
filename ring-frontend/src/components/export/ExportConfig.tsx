import { useState, useEffect } from 'react'
import type { ExportTypeInfo, ExportFormat } from '../../types'
import type { GraphNode, Conversation, SessionListItem } from '../../types'
import * as api from '../../api/client'
import { NodePicker } from './NodePicker'
import './ExportConfig.css'

interface ExportConfigProps {
  type_info: ExportTypeInfo
  ring_id: string
  exporting: boolean
  error: string | null
  success: boolean
  on_export: (config: Record<string, unknown>) => void
}

export function ExportConfig({ type_info, ring_id, exporting, error, success, on_export }: ExportConfigProps) {
  const [format, set_format] = useState<ExportFormat>(type_info.formats[0])
  const [graph_id, set_graph_id] = useState('')
  const [graphs, set_graphs] = useState<string[]>([])
  const [nodes, set_nodes] = useState<GraphNode[]>([])
  const [node_id, set_node_id] = useState('')
  const [node_ids, set_node_ids] = useState<string[]>([])
  const [conversations, set_conversations] = useState<Conversation[]>([])
  const [conversation_id, set_conversation_id] = useState('')
  const [sessions, set_sessions] = useState<SessionListItem[]>([])
  const [session_id, set_session_id] = useState('')
  const [topic, set_topic] = useState('')
  const [include_ai, set_include_ai] = useState(true)

  useEffect(() => {
    if (type_info.type === 'graph_image' || type_info.type === 'graph_json' || type_info.type === 'report') {
      api.list_graphs(ring_id).then(set_graphs).catch(() => {})
    }
    if (type_info.type === 'markdown' || type_info.type === 'report') {
      api.list_graphs(ring_id).then((gids) => {
        if (gids.length > 0) {
          set_graph_id(gids[0])
          api.get_graph(ring_id, gids[0]).then((g) => set_nodes(g.nodes)).catch(() => {})
        }
        set_graphs(gids)
      }).catch(() => {})
    }
    if (type_info.type === 'conversation') {
      api.list_conversations(ring_id).then(set_conversations).catch(() => {})
    }
    if (type_info.type === 'session') {
      api.list_sessions(ring_id).then(set_sessions).catch(() => {})
    }
  }, [type_info.type, ring_id])

  useEffect(() => { set_format(type_info.formats[0]) }, [type_info])

  const handle_graph_change = (gid: string) => {
    set_graph_id(gid)
    api.get_graph(ring_id, gid).then((g) => set_nodes(g.nodes)).catch(() => {})
  }

  const build_config = (): Record<string, unknown> => {
    const cfg: Record<string, unknown> = { format }
    switch (type_info.type) {
      case 'graph_image':
      case 'graph_json':
        return { ...cfg, graph_id }
      case 'markdown':
        return { ...cfg, node_id }
      case 'conversation':
        return { ...cfg, conversation_id, include_ai_responses: include_ai }
      case 'session':
        return { ...cfg, session_id }
      case 'report':
        return { ...cfg, topic, node_ids, graph_id }
      case 'backup':
        return cfg
    }
    return cfg
  }

  const can_export = (): boolean => {
    switch (type_info.type) {
      case 'graph_image':
      case 'graph_json':
        return !!graph_id
      case 'markdown':
        return !!node_id
      case 'conversation':
        return !!conversation_id
      case 'session':
        return !!session_id
      case 'report':
        return !!topic && node_ids.length > 0 && !!graph_id
      case 'backup':
        return true
    }
    return false
  }

  return (
    <div className="export-config">
      <div className="export-config-title">{type_info.icon} {type_info.label}</div>
      <div className="export-config-desc">{type_info.description}</div>

      {type_info.formats.length > 1 && (
        <div className="export-config-field">
          <label>格式</label>
          <div className="export-format-options">
            {type_info.formats.map((f) => (
              <button
                key={f}
                className={`export-format-btn${format === f ? ' export-format-active' : ''}`}
                onClick={() => set_format(f)}
              >
                {f.toUpperCase()}
              </button>
            ))}
          </div>
        </div>
      )}

      {(type_info.type === 'graph_image' || type_info.type === 'graph_json') && (
        <div className="export-config-field">
          <label>选择图谱</label>
          <select value={graph_id} onChange={(e) => set_graph_id(e.target.value)}>
            <option value="">-- 选择 --</option>
            {graphs.map((gid) => <option key={gid} value={gid}>{gid}</option>)}
          </select>
        </div>
      )}

      {type_info.type === 'markdown' && (
        <div className="export-config-field">
          <label>选择图谱</label>
          <select value={graph_id} onChange={(e) => handle_graph_change(e.target.value)}>
            <option value="">-- 选择 --</option>
            {graphs.map((gid) => <option key={gid} value={gid}>{gid}</option>)}
          </select>
          {nodes.length > 0 && (
            <NodePicker nodes={nodes} selected={node_id} on_select={set_node_id} multiple={false} />
          )}
        </div>
      )}

      {type_info.type === 'conversation' && (
        <>
          <div className="export-config-field">
            <label>选择对话</label>
            <select value={conversation_id} onChange={(e) => set_conversation_id(e.target.value)}>
              <option value="">-- 选择 --</option>
              {conversations.map((c) => <option key={c.id} value={c.id}>{c.title || c.id}</option>)}
            </select>
          </div>
          <div className="export-config-field">
            <label>
              <input type="checkbox" checked={include_ai} onChange={(e) => set_include_ai(e.target.checked)} />
              包含 AI 回复
            </label>
          </div>
        </>
      )}

      {type_info.type === 'session' && (
        <div className="export-config-field">
          <label>选择 Session</label>
          <select value={session_id} onChange={(e) => set_session_id(e.target.value)}>
            <option value="">-- 选择 --</option>
            {sessions.map((s) => <option key={s.id} value={s.id}>{s.title || s.id}</option>)}
          </select>
        </div>
      )}

      {type_info.type === 'report' && (
        <>
          <div className="export-config-field">
            <label>选择图谱</label>
            <select value={graph_id} onChange={(e) => handle_graph_change(e.target.value)}>
              <option value="">-- 选择 --</option>
              {graphs.map((gid) => <option key={gid} value={gid}>{gid}</option>)}
            </select>
          </div>
          {nodes.length > 0 && (
            <div className="export-config-field">
              <label>选择节点（多选）</label>
              <NodePicker nodes={nodes} selected={node_ids} on_select={set_node_ids} multiple={true} />
            </div>
          )}
          <div className="export-config-field">
            <label>报告主题</label>
            <input type="text" value={topic} onChange={(e) => set_topic(e.target.value)} placeholder="输入报告主题..." />
          </div>
        </>
      )}

      {type_info.type === 'backup' && (
        <div className="export-config-warning">
          将导出 Ring 全部数据，包括图谱、文档、对话历史和资源文件。
        </div>
      )}

      {error && <div className="export-config-error">{error}</div>}
      {success && <div className="export-config-success">导出成功，文件已下载</div>}

      <button
        className="export-config-submit"
        disabled={!can_export() || exporting}
        onClick={() => on_export(build_config())}
      >
        {exporting ? '导出中...' : '导出'}
      </button>
    </div>
  )
}
