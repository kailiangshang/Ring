import { useEffect, useRef, useState } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { useGraphStore } from '../../stores/graph-store'
import {
  useBlueprintStore,
  stripBlueprintTags,
  type BlueprintGraph,
} from '../../stores/blueprint-store'
import { api, parseFile } from '../../services/api'
import * as d3 from 'd3'

interface BlueprintPreview {
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

const TEMPLATES = [
  { id: 'product-research', name: '竞品分析', desc: '产品分析和竞品调研', icon: '📊' },
  { id: 'project-management', name: '项目管理', desc: '项目规划和进度跟踪', icon: '📋' },
  { id: 'learning-notes', name: '学习笔记', desc: '知识学习和笔记整理', icon: '📚' },
  { id: 'technical-docs', name: '技术文档', desc: '技术方案设计和文档', icon: '🔧' },
  { id: 'blank', name: '空白', desc: '从零开始构建图谱', icon: '✦' },
]

function MiniGraphPreview({ graphs }: { graphs: BlueprintGraph[] }) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current || graphs.length === 0) return
    let graph: BlueprintGraph | undefined
    try {
      graph = graphs[0]
      if (!graph || !Array.isArray(graph.nodes) || !Array.isArray(graph.edges)) return
    } catch { return }

    const container = ref.current
    const width = container.clientWidth || 280
    const height = 160

    d3.select(container).selectAll('*').remove()

    const svg = d3
      .select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', height)

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

    interface SimNode extends d3.SimulationNodeDatum {
      id: number
      label: string
      node_type: string
    }

    interface SimLink extends d3.SimulationLinkDatum<SimNode> {
      relation: string
    }

    const simNodes: SimNode[] = nodes as SimNode[]
    const simEdges: SimLink[] = edges as SimLink[]

    let simulation: d3.Simulation<SimNode, SimLink>
    try {
      simulation = d3
        .forceSimulation(simNodes)
        .force(
          'link',
          d3
            .forceLink<SimNode, SimLink>(simEdges)
            .id((d) => d.id)
            .distance(50),
        )
        .force('charge', d3.forceManyBody().strength(-120))
        .force('center', d3.forceCenter(width / 2, height / 2))
    } catch (err) {
      console.error('MiniGraphPreview: simulation creation failed', err)
      return
    }

    const link = svg
      .append('g')
      .selectAll('line')
      .data(simEdges)
      .join('line')
      .attr('stroke', '#475569')
      .attr('stroke-width', 1)
      .attr('stroke-opacity', 0.6)

    const node = svg
      .append('g')
      .selectAll('circle')
      .data(simNodes)
      .join('circle')
      .attr('r', 6)
      .attr('fill', (d) => colorMap[d.node_type] || '#94a3b8')

    const label = svg
      .append('g')
      .selectAll('text')
      .data(simNodes)
      .join('text')
      .text((d) => d.label)
      .attr('font-size', 7)
      .attr('fill', '#94a3b8')
      .attr('text-anchor', 'middle')
      .attr('dy', -10)

    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as SimNode).x ?? 0)
        .attr('y1', (d) => (d.source as SimNode).y ?? 0)
        .attr('x2', (d) => (d.target as SimNode).x ?? 0)
        .attr('y2', (d) => (d.target as SimNode).y ?? 0)
      node.attr('cx', (d) => d.x ?? 0).attr('cy', (d) => d.y ?? 0)
      label.attr('x', (d) => d.x ?? 0).attr('y', (d) => d.y ?? 0)
    })
  }, [graphs])

  return <div ref={ref} style={{ width: '100%', height: 160 }} />
}

export function BlueprintPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
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
  const step = useBlueprintStore((s) => s.step)
  const setStep = useBlueprintStore((s) => s.setStep)

  const [input, setInput] = useState('')
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null)
  const [preview, setPreview] = useState<BlueprintPreview | null>(null)
  const [loading, setLoading] = useState(false)
  const [uploadedContext, setUploadedContext] = useState<string>('')
  const chatEndRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (active_ring_id) {
      checkStatus(active_ring_id)
      loadHistory(active_ring_id)
    }
  }, [active_ring_id, checkStatus, loadHistory])

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  useEffect(() => {
    if (current_blueprint && current_blueprint.graphs.length > 0 && step === 'template') {
      setStep('confirm')
    }
  }, [current_blueprint, step, setStep])

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

  const handleTemplateSelect = async (templateId: string) => {
    setSelectedTemplate(templateId)
    if (templateId === 'blank') {
      setStep('refine')
      return
    }
    try {
      const res = await api.post<{ preview: BlueprintPreview }>(
        `/rings/${active_ring_id}/blueprint/from-template`,
        { template: templateId },
      )
      setPreview(res.preview)
    } catch {
      // silently ignore
    }
  }

  const handleTemplateConfirm = () => {
    if (preview && preview.nodes.length > 0) {
      setStep('refine')
      if (uploadedContext) {
        const contextMsg = `I have these reference materials:\n\n${uploadedContext.slice(0, 3000)}\n\nBased on the selected template and these materials, please refine the knowledge graph blueprint.`
        setTimeout(() => {
          sendMessage(active_ring_id, contextMsg)
        }, 100)
      }
    } else {
      setStep('refine')
    }
  }

  const handleFinalConfirm = async () => {
    setLoading(true)
    try {
      if (preview && !current_blueprint) {
        const blueprintData = {
          graphs: [{
            name: 'Main',
            nodes: preview.nodes,
            edges: preview.edges,
          }],
        }
        await api.post(`/rings/${active_ring_id}/blueprint/confirm`, {
          blueprint: blueprintData,
        })
      } else {
        await confirm(active_ring_id)
      }
      fetchGraph(active_ring_id)
      useBlueprintStore.setState({ confirmed: true })
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'unknown error'
      alert(`Failed to confirm blueprint: ${msg}`)
    }
    setLoading(false)
  }

  const handleSend = () => {
    const trimmed = input.trim()
    if (!trimmed || !active_ring_id) return
    sendMessage(active_ring_id, trimmed)
    setInput('')
  }

  const handleFileUpload = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    for (let i = 0; i < files.length; i++) {
      try {
        const parsed = await parseFile(files[i])
        setUploadedContext(prev =>
          prev ? `${prev}\n\n--- ${parsed.filename} ---\n${parsed.content.slice(0, 5000)}` : `--- ${parsed.filename} ---\n${parsed.content.slice(0, 5000)}`
        )
      } catch {
        // skip failed files
      }
    }
  }

  const stepIndicator = (
    <div style={{ display: 'flex', gap: 2, marginBottom: 12 }}>
      {(['template', 'refine', 'confirm'] as const).map((s, i) => (
        <div
          key={s}
          style={{
            flex: 1,
            height: 3,
            borderRadius: 2,
            background: step === s ? 'var(--accent-cyan)' :
              (['template', 'refine', 'confirm'].indexOf(step) > i) ? 'var(--accent-cyan)' : 'var(--border)',
            opacity: step === s ? 1 : (['template', 'refine', 'confirm'].indexOf(step) > i) ? 0.5 : 0.3,
          }}
        />
      ))}
    </div>
  )

  return (
    <div style={{ padding: '12px 16px', height: '100%', display: 'flex', flexDirection: 'column' }}>
      {stepIndicator}

      {step === 'template' && (
        <div style={{ flex: 1, overflow: 'auto' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 8 }}>
            1. Choose a template
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4, marginBottom: 10 }}>
            {TEMPLATES.map((t) => (
              <button
                key={t.id}
                onClick={() => handleTemplateSelect(t.id)}
                style={{
                  padding: '8px 10px',
                  background: selectedTemplate === t.id ? 'var(--bg-hover)' : 'var(--bg-input)',
                  border: `1px solid ${selectedTemplate === t.id ? 'var(--accent-cyan)' : 'var(--border)'}`,
                  borderRadius: 4,
                  cursor: 'pointer',
                  textAlign: 'left',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <span style={{ fontSize: 16 }}>{t.icon}</span>
                <div>
                  <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)' }}>{t.name}</div>
                  <div style={{ fontSize: 9, color: 'var(--text-dim)' }}>{t.desc}</div>
                </div>
              </button>
            ))}
          </div>

          {preview && (
            <div style={{ background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: 10, marginBottom: 10 }}>
              <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 6 }}>
                Preview: {preview.nodes.length} nodes · {preview.edges.length} edges
              </div>
              <MiniGraphPreview graphs={[{ name: 'Main', nodes: preview.nodes, edges: preview.edges }]} />
            </div>
          )}

          <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 6 }}>
            Upload reference docs (optional)
          </div>
          <div style={{ display: 'flex', gap: 6, alignItems: 'center', marginBottom: 10 }}>
            <input
              ref={fileInputRef}
              type="file"
              multiple
              accept=".txt,.md,.csv,.json,.pdf"
              style={{ display: 'none' }}
              onChange={(e) => handleFileUpload(e.target.files)}
            />
            <button
              onClick={() => fileInputRef.current?.click()}
              style={{
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '5px 10px',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              📎 Upload
            </button>
            {uploadedContext && (
              <span style={{ fontSize: 9, color: 'var(--accent-cyan)' }}>
                ✓ {uploadedContext.split('---').length - 1} file(s) loaded
              </span>
            )}
          </div>

          <button
            onClick={handleTemplateConfirm}
            disabled={!selectedTemplate}
            style={{
              width: '100%',
              padding: '8px',
              background: selectedTemplate ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: selectedTemplate ? 'var(--bg-base)' : 'var(--text-dim)',
              border: 'none',
              borderRadius: 4,
              fontSize: 11,
              fontWeight: 700,
              cursor: selectedTemplate ? 'pointer' : 'default',
            }}
          >
            Next →
          </button>
        </div>
      )}

      {step === 'refine' && (
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-primary)' }}>
              2. Refine with AI
            </div>
            {preview && !current_blueprint && (
              <button
                onClick={() => setStep('confirm')}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--accent-cyan)',
                  fontSize: 10,
                  cursor: 'pointer',
                  fontWeight: 700,
                }}
              >
                Skip →
              </button>
            )}
          </div>

          {preview && !current_blueprint && (
            <div style={{ background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: 8, marginBottom: 8, maxHeight: 80, overflowY: 'auto' }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', marginBottom: 4 }}>Starting blueprint:</div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3 }}>
                {preview.nodes.map((n, i) => (
                  <span key={i} style={{ fontSize: 9, background: 'var(--bg-hover)', padding: '1px 5px', borderRadius: 2, color: 'var(--text-secondary)' }}>
                    {n.label}
                  </span>
                ))}
              </div>
            </div>
          )}

          <div style={{ flex: 1, overflowY: 'auto', marginBottom: 8, minHeight: 0 }}>
            {messages.length === 0 && (
              <div style={{ fontSize: 10, color: 'var(--text-dim)', padding: '16px 0', textAlign: 'center' }}>
                Describe your knowledge domain to refine the blueprint
              </div>
            )}
            {messages.map((msg, i) => (
              <div
                key={msg.id || i}
                style={{
                  marginBottom: 6,
                  display: 'flex',
                  justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
                }}
              >
                <div style={{
                  maxWidth: '85%',
                  padding: '6px 8px',
                  borderRadius: 4,
                  fontSize: 10,
                  lineHeight: 1.5,
                  background: msg.role === 'user' ? 'var(--bg-hover)' : 'var(--bg-input)',
                  color: 'var(--text-secondary)',
                  border: `1px solid ${msg.role === 'user' ? 'var(--accent-cyan)' : 'var(--border)'}`,
                }}>
                  {msg.role === 'blueprint'
                    ? stripBlueprintTags(msg.content) || '(Blueprint updated)'
                    : msg.content}
                </div>
              </div>
            ))}
            <div ref={chatEndRef} />
          </div>

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
              placeholder="Describe your knowledge domain..."
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
                  background: 'var(--accent-amber)',
                  borderRadius: 4,
                  color: 'var(--bg-base)',
                  border: 'none',
                  padding: '4px 8px',
                  fontSize: 9,
                  fontWeight: 700,
                  cursor: 'pointer',
                }}
              >
                STOP
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                style={{
                  background: input.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  borderRadius: 4,
                  color: input.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
                  border: 'none',
                  padding: '4px 8px',
                  fontSize: 9,
                  fontWeight: 700,
                  cursor: input.trim() ? 'pointer' : 'default',
                }}
              >
                SEND
              </button>
            )}
          </div>
        </div>
      )}

      {step === 'confirm' && (
        <div style={{ flex: 1, overflow: 'auto' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 8 }}>
            3. Preview & Confirm
          </div>

          {(current_blueprint && current_blueprint.graphs.length > 0) ? (
            <div>
              <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4 }}>
                AI-generated blueprint: {current_blueprint.graphs.reduce((a, g) => a + g.nodes.length, 0)} nodes · {current_blueprint.graphs.reduce((a, g) => a + g.edges.length, 0)} edges
              </div>
              <MiniGraphPreview graphs={current_blueprint.graphs} />
            </div>
          ) : preview ? (
            <div>
              <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4 }}>
                Template blueprint: {preview.nodes.length} nodes · {preview.edges.length} edges
              </div>
              <MiniGraphPreview graphs={[{ name: 'Main', nodes: preview.nodes, edges: preview.edges }]} />
            </div>
          ) : (
            <div style={{ fontSize: 10, color: 'var(--text-dim)', textAlign: 'center', padding: 20 }}>
              No blueprint to preview. Go back and create one.
            </div>
          )}

          <div style={{ display: 'flex', gap: 6, marginTop: 12 }}>
            <button
              onClick={() => setStep('refine')}
              style={{
                flex: 1,
                padding: '8px',
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                fontSize: 10,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              ← Back
            </button>
            <button
              onClick={handleFinalConfirm}
              disabled={loading || (!preview && !current_blueprint)}
              style={{
                flex: 2,
                padding: '8px',
                background: (preview || current_blueprint) ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: (preview || current_blueprint) ? 'var(--bg-base)' : 'var(--text-dim)',
                border: 'none',
                borderRadius: 4,
                fontSize: 11,
                fontWeight: 700,
                cursor: (preview || current_blueprint) && !loading ? 'pointer' : 'default',
              }}
            >
              {loading ? 'Creating...' : 'Confirm Blueprint'}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
