import { useEffect, useRef, useState } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { useGraphStore } from '../../stores/graph-store'
import {
  useBlueprintStore,
  stripBlueprintTags,
  type BlueprintGraph,
} from '../../stores/blueprint-store'
import { api } from '../../services/api'
import * as d3 from 'd3'

interface BlueprintPreview {
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

const TEMPLATES = [
  { id: 'product-research', name: '竞品分析', desc: '产品分析和竞品调研' },
  { id: 'project-management', name: '项目管理', desc: '项目规划和进度跟踪' },
  { id: 'learning-notes', name: '学习笔记', desc: '知识学习和笔记整理' },
  { id: 'technical-docs', name: '技术文档', desc: '技术方案设计和文档' },
  { id: 'blank', name: '空白', desc: '从零开始构建图谱' },
]

function MiniGraphPreview({ graphs }: { graphs: BlueprintGraph[] }) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current || graphs.length === 0) return
    const container = ref.current
    const width = container.clientWidth || 280
    const height = 160

    d3.select(container).selectAll('*').remove()

    const svg = d3
      .select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', height)

    const graph = graphs[0]
    const labels = graph.nodes.map((n) => n.label)
    const nodeMap = new Map(labels.map((l, i) => [l, i]))

    const nodes = graph.nodes.map((n, i) => ({
      id: i,
      label: n.label,
      node_type: n.node_type,
    }))

    const edges = graph.edges
      .filter((e) => nodeMap.has(e.from) && nodeMap.has(e.to))
      .map((e) => ({
        source: nodeMap.get(e.from)!,
        target: nodeMap.get(e.to)!,
        relation: e.relation,
      }))

    const colorMap: Record<string, string> = {
      category: '#22d3ee',
      topic: '#a78bfa',
      leaf: '#34d399',
    }

    const simulation = d3
      .forceSimulation(nodes as d3.SimulationNodeDatum[])
      .force(
        'link',
        d3
          .forceLink(edges as d3.SimulationLinkDatum<d3.SimulationNodeDatum>[])
          .id((d: any) => d.id)
          .distance(50),
      )
      .force('charge', d3.forceManyBody().strength(-120))
      .force('center', d3.forceCenter(width / 2, height / 2))

    const link = svg
      .append('g')
      .selectAll('line')
      .data(edges)
      .join('line')
      .attr('stroke', '#475569')
      .attr('stroke-width', 1)
      .attr('stroke-opacity', 0.6)

    const node = svg
      .append('g')
      .selectAll('circle')
      .data(nodes)
      .join('circle')
      .attr('r', 6)
      .attr('fill', (d: any) => colorMap[d.node_type] || '#94a3b8')

    const label = svg
      .append('g')
      .selectAll('text')
      .data(nodes)
      .join('text')
      .text((d: any) => d.label)
      .attr('font-size', 7)
      .attr('fill', '#94a3b8')
      .attr('text-anchor', 'middle')
      .attr('dy', -10)

    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y)
      node.attr('cx', (d: any) => d.x).attr('cy', (d: any) => d.y)
      label.attr('x', (d: any) => d.x).attr('y', (d: any) => d.y)
    })
  }, [graphs])

  return <div ref={ref} style={{ width: '100%', height: 160 }} />
}

export function BlueprintPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const mode = useBlueprintStore((s) => s.mode)
  const setMode = useBlueprintStore((s) => s.setMode)
  const messages = useBlueprintStore((s) => s.messages)
  const streaming = useBlueprintStore((s) => s.streaming)
  const current_blueprint = useBlueprintStore((s) => s.current_blueprint)
  const confirmed = useBlueprintStore((s) => s.confirmed)
  const sendMessage = useBlueprintStore((s) => s.sendMessage)
  const confirm = useBlueprintStore((s) => s.confirm)
  const loadHistory = useBlueprintStore((s) => s.loadHistory)
  const checkStatus = useBlueprintStore((s) => s.checkStatus)
  const stopStreaming = useBlueprintStore((s) => s.stopStreaming)
  const fetchGraph = useGraphStore((s) => s.fetchGraph)

  const [input, setInput] = useState('')
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null)
  const [preview, setPreview] = useState<BlueprintPreview | null>(null)
  const [loading, setLoading] = useState(false)
  const chatEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (active_ring_id) {
      checkStatus(active_ring_id)
      if (mode === 'deep') loadHistory(active_ring_id)
    }
  }, [active_ring_id, mode])

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  if (!active_ring_id) return null

  if (confirmed) {
    return (
      <div style={{ padding: 20, textAlign: 'center' }}>
        <div style={{ fontSize: 48, marginBottom: 16 }}>&#10003;</div>
        <h2 style={{ color: 'var(--accent-green)', marginBottom: 8 }}>
          Blueprint Confirmed
        </h2>
        <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
          Your ring blueprint has been set up. You can now start building your
          knowledge graph.
        </p>
      </div>
    )
  }

  const handleQuickConfirm = async () => {
    if (!active_ring_id || !preview) return
    setLoading(true)
    try {
      const blueprintData = preview.nodes.length > 0
        ? {
            graphs: [
              {
                name: 'Main',
                nodes: preview.nodes,
                edges: preview.edges,
              },
            ],
          }
        : null
      await api.post(`/rings/${active_ring_id}/blueprint/confirm`, {
        blueprint: blueprintData,
      })
      fetchGraph(active_ring_id)
      useBlueprintStore.setState({ confirmed: true })
    } catch (e: any) {
      alert(`Failed to confirm blueprint: ${e.message || 'unknown error'}`)
    }
    setLoading(false)
  }

  const handleDeepConfirm = async () => {
    if (!active_ring_id) return
    setLoading(true)
    await confirm(active_ring_id)
    fetchGraph(active_ring_id)
    setLoading(false)
  }

  const handleSend = () => {
    const trimmed = input.trim()
    if (!trimmed || !active_ring_id) return
    sendMessage(active_ring_id, trimmed)
    setInput('')
  }

  const btnBase: React.CSSProperties = {
    background: 'none',
    border: 'none',
    fontSize: 11,
    fontWeight: 700,
    cursor: 'pointer',
    padding: '2px 6px',
  }

  return (
    <div
      style={{
        padding: '12px 16px',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        <button
          onClick={() => setMode('quick')}
          style={{
            flex: 1,
            padding: '6px 8px',
            fontSize: 10,
            fontWeight: 700,
            background: mode === 'quick' ? 'var(--bg-hover)' : 'var(--bg-input)',
            border: `1px solid ${mode === 'quick' ? 'var(--accent-cyan)' : 'var(--border)'}`,
            borderRadius: 4,
            color: mode === 'quick' ? 'var(--accent-cyan)' : 'var(--text-dim)',
            cursor: 'pointer',
          }}
        >
          从模板选择
        </button>
        <button
          onClick={() => setMode('deep')}
          style={{
            flex: 1,
            padding: '6px 8px',
            fontSize: 10,
            fontWeight: 700,
            background: mode === 'deep' ? 'var(--bg-hover)' : 'var(--bg-input)',
            border: `1px solid ${mode === 'deep' ? 'var(--accent-cyan)' : 'var(--border)'}`,
            borderRadius: 4,
            color: mode === 'deep' ? 'var(--accent-cyan)' : 'var(--text-dim)',
            cursor: 'pointer',
          }}
        >
          AI 协作设计
        </button>
      </div>

      {mode === 'quick' && (
        <div style={{ flex: 1, overflow: 'auto' }}>
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: 6,
              marginBottom: 12,
            }}
          >
            {TEMPLATES.map((t) => (
              <button
                key={t.id}
                onClick={async () => {
                  setSelectedTemplate(t.id)
                  try {
                    const res = await api.post<{ preview: BlueprintPreview }>(
                      `/rings/${active_ring_id}/blueprint/from-template`,
                      { template: t.id },
                    )
                    setPreview(res.preview)
                  } catch {
      // silently ignore
    }
                }}
                style={{
                  padding: '8px 10px',
                  background:
                    selectedTemplate === t.id
                      ? 'var(--bg-hover)'
                      : 'var(--bg-input)',
                  border: `1px solid ${selectedTemplate === t.id ? 'var(--accent-cyan)' : 'var(--border)'}`,
                  borderRadius: 4,
                  cursor: 'pointer',
                  textAlign: 'left',
                }}
              >
                <div
                  style={{
                    fontSize: 11,
                    fontWeight: 700,
                    color: 'var(--text-primary)',
                  }}
                >
                  {t.name}
                </div>
                <div
                  style={{ fontSize: 9, color: 'var(--text-dim)', marginTop: 2 }}
                >
                  {t.desc}
                </div>
              </button>
            ))}
          </div>

          {preview && (
            <div
              style={{
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: 10,
                marginBottom: 10,
              }}
            >
              <div
                style={{
                  fontSize: 10,
                  color: 'var(--text-dim)',
                  marginBottom: 4,
                }}
              >
                预览: {preview.nodes.length} 节点 · {preview.edges.length} 边
              </div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                {preview.nodes.map((n, i) => (
                  <span
                    key={i}
                    style={{
                      fontSize: 9,
                      background: 'var(--bg-hover)',
                      padding: '2px 5px',
                      borderRadius: 2,
                      color: 'var(--text-secondary)',
                    }}
                  >
                    {n.label}
                  </span>
                ))}
              </div>
            </div>
          )}

          {selectedTemplate && (
            <button
              onClick={handleQuickConfirm}
              disabled={loading}
              style={{
                width: '100%',
                padding: '8px',
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 4,
                fontSize: 11,
                fontWeight: 700,
                cursor: loading ? 'default' : 'pointer',
              }}
            >
              {loading ? '创建中...' : '确认并应用蓝图'}
            </button>
          )}
        </div>
      )}

      {mode === 'deep' && (
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            minHeight: 0,
          }}
        >
          <div
            style={{
              flex: 1,
              overflowY: 'auto',
              marginBottom: 8,
              minHeight: 0,
            }}
          >
            {messages.length === 0 && (
              <div
                style={{
                  fontSize: 10,
                  color: 'var(--text-dim)',
                  padding: '20px 0',
                  textAlign: 'center',
                }}
              >
                描述你的知识领域，AI 会帮你设计图谱结构
              </div>
            )}
            {messages.map((msg, i) => (
              <div
                key={msg.id || i}
                style={{
                  marginBottom: 6,
                  display: 'flex',
                  justifyContent:
                    msg.role === 'user' ? 'flex-end' : 'flex-start',
                }}
              >
                <div
                  style={{
                    maxWidth: '85%',
                    padding: '6px 8px',
                    borderRadius: 4,
                    fontSize: 10,
                    lineHeight: 1.5,
                    background:
                      msg.role === 'user'
                        ? 'var(--bg-hover)'
                        : 'var(--bg-input)',
                    color: 'var(--text-secondary)',
                    border: `1px solid ${msg.role === 'user' ? 'var(--accent-cyan)' : 'var(--border)'}`,
                  }}
                >
                  {msg.role === 'blueprint'
                    ? stripBlueprintTags(msg.content) || '(图谱已更新)'
                    : msg.content}
                </div>
              </div>
            ))}
            <div ref={chatEndRef} />
          </div>

          {current_blueprint && current_blueprint.graphs.length > 0 && (
            <div
              style={{
                borderTop: '1px solid var(--border)',
                paddingTop: 6,
                marginBottom: 6,
              }}
            >
              <div
                style={{
                  fontSize: 9,
                  color: 'var(--text-dim)',
                  marginBottom: 4,
                }}
              >
                当前蓝图: {current_blueprint.graphs.length} 个图谱
              </div>
              <MiniGraphPreview graphs={current_blueprint.graphs} />
            </div>
          )}

          {current_blueprint && current_blueprint.graphs.length > 0 && (
            <button
              onClick={handleDeepConfirm}
              disabled={loading}
              style={{
                width: '100%',
                padding: '6px',
                marginBottom: 6,
                background: 'var(--accent-green)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 4,
                fontSize: 10,
                fontWeight: 700,
                cursor: loading ? 'default' : 'pointer',
              }}
            >
              {loading ? '创建中...' : '确认蓝图'}
            </button>
          )}

          <div style={{ display: 'flex', gap: 6 }}>
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  handleSend()
                }
              }}
              placeholder="描述你的知识领域..."
              style={{
                flex: 1,
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '6px 8px',
                color: 'var(--text-primary)',
                fontSize: 10,
                resize: 'none',
                fontFamily: 'inherit',
                outline: 'none',
                minHeight: 28,
                maxHeight: 60,
              }}
            />
            {streaming ? (
              <button
                onClick={stopStreaming}
                style={{
                  ...btnBase,
                  background: 'var(--accent-amber)',
                  borderRadius: 4,
                  color: 'var(--bg-base)',
                  padding: '4px 8px',
                  fontSize: 9,
                }}
              >
                STOP
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                style={{
                  ...btnBase,
                  background: 'var(--accent-cyan)',
                  borderRadius: 4,
                  color: 'var(--bg-base)',
                  opacity: input.trim() ? 1 : 0.5,
                  padding: '4px 8px',
                  fontSize: 9,
                }}
              >
                SEND
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
