import { useEffect, useState } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { api } from '../../services/api'

interface BlueprintTemplate {
  id: string
  name: string
  description: string
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

interface BlueprintPreview {
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

const TEMPLATES: BlueprintTemplate[] = [
  {
    id: 'product-research',
    name: 'Product Research',
    description: 'For product analysis and competitor research',
    nodes: [
      { label: 'Product Overview', node_type: 'category', tags: ['overview'] },
      { label: 'Competitor A', node_type: 'topic', tags: ['competitor'] },
      { label: 'Competitor B', node_type: 'topic', tags: ['competitor'] },
      { label: 'Market Trends', node_type: 'topic', tags: ['market'] },
      { label: 'Feature Comparison', node_type: 'topic', tags: ['comparison'] },
      { label: 'User Feedback', node_type: 'topic', tags: ['feedback'] },
      { label: 'Decision Log', node_type: 'topic', tags: ['decision'] },
    ],
    edges: [
      { from: 'Product Overview', to: 'Competitor A', relation: 'contains' },
      { from: 'Product Overview', to: 'Competitor B', relation: 'contains' },
      { from: 'Product Overview', to: 'Market Trends', relation: 'contains' },
      { from: 'Competitor A', to: 'Feature Comparison', relation: 'relates_to' },
      { from: 'Competitor B', to: 'Feature Comparison', relation: 'relates_to' },
      { from: 'Market Trends', to: 'Decision Log', relation: 'influences' },
      { from: 'User Feedback', to: 'Decision Log', relation: 'influences' },
    ],
  },
  {
    id: 'project-management',
    name: 'Project Management',
    description: 'For project planning and tracking',
    nodes: [
      { label: 'Project Goals', node_type: 'category', tags: ['goal'] },
      { label: 'Requirements', node_type: 'topic', tags: ['requirement'] },
      { label: 'Tech Stack', node_type: 'topic', tags: ['tech'] },
      { label: 'Task List', node_type: 'topic', tags: ['task'] },
      { label: 'Milestones', node_type: 'topic', tags: ['milestone'] },
      { label: 'Risks', node_type: 'topic', tags: ['risk'] },
      { label: 'Meetings', node_type: 'topic', tags: ['meeting'] },
    ],
    edges: [
      { from: 'Project Goals', to: 'Requirements', relation: 'depends_on' },
      { from: 'Requirements', to: 'Tech Stack', relation: 'derives_from' },
      { from: 'Tech Stack', to: 'Task List', relation: 'contains' },
      { from: 'Task List', to: 'Milestones', relation: 'leads_to' },
      { from: 'Risks', to: 'Task List', relation: 'affects' },
    ],
  },
  {
    id: 'learning-notes',
    name: 'Learning Notes',
    description: 'For knowledge learning and note-taking',
    nodes: [
      { label: 'Learning Topic', node_type: 'category', tags: ['topic'] },
      { label: 'Core Concepts', node_type: 'topic', tags: ['concept'] },
      { label: 'References', node_type: 'topic', tags: ['reference'] },
      { label: 'Examples', node_type: 'topic', tags: ['example'] },
      { label: 'Questions', node_type: 'topic', tags: ['question'] },
      { label: 'Summary', node_type: 'topic', tags: ['summary'] },
    ],
    edges: [
      { from: 'Learning Topic', to: 'Core Concepts', relation: 'contains' },
      { from: 'Core Concepts', to: 'References', relation: 'documented_in' },
      { from: 'Core Concepts', to: 'Examples', relation: 'illustrated_by' },
      { from: 'Examples', to: 'Questions', relation: 'raises' },
      { from: 'Questions', to: 'Summary', relation: 'resolved_in' },
    ],
  },
  {
    id: 'technical-docs',
    name: 'Technical Docs',
    description: 'For technical design and documentation',
    nodes: [
      { label: 'System Architecture', node_type: 'category', tags: ['architecture'] },
      { label: 'API Design', node_type: 'topic', tags: ['api'] },
      { label: 'Data Model', node_type: 'topic', tags: ['data'] },
      { label: 'Deployment', node_type: 'topic', tags: ['deploy'] },
      { label: 'Performance', node_type: 'topic', tags: ['performance'] },
      { label: 'Troubleshooting', node_type: 'topic', tags: ['troubleshoot'] },
      { label: 'Changelog', node_type: 'topic', tags: ['changelog'] },
    ],
    edges: [
      { from: 'System Architecture', to: 'API Design', relation: 'contains' },
      { from: 'System Architecture', to: 'Data Model', relation: 'contains' },
      { from: 'API Design', to: 'Deployment', relation: 'requires' },
      { from: 'Deployment', to: 'Performance', relation: 'measured_by' },
      { from: 'Troubleshooting', to: 'Changelog', relation: 'documented_in' },
    ],
  },
  {
    id: 'blank',
    name: 'Blank',
    description: 'Start from scratch',
    nodes: [
      { label: 'Central Topic', node_type: 'category', tags: [] },
    ],
    edges: [],
  },
]

export function BlueprintPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null)
  const [preview, setPreview] = useState<BlueprintPreview | null>(null)
  const [loading, setLoading] = useState(false)
  const [confirmed, setConfirmed] = useState(false)
  const createNode = useGraphStore((s) => s.createNode)

  useEffect(() => {
    if (active_ring_id) {
      api.get(`/rings/${active_ring_id}/blueprint`)
        .then((res: any) => {
          if (res.status === 'confirmed') {
            setConfirmed(true)
          }
        })
        .catch(() => {})
    }
  }, [active_ring_id])

  const handlePreview = async (templateId: string) => {
    if (!active_ring_id) return
    setSelectedTemplate(templateId)
    setLoading(true)
    try {
      const res = await api.post<{ preview: BlueprintPreview }>(`/rings/${active_ring_id}/blueprint/from-template`, {
        template: templateId,
      })
      setPreview(res.preview)
    } catch {
      const template = TEMPLATES.find((t) => t.id === templateId)
      if (template) {
        setPreview({
          nodes: template.nodes,
          edges: template.edges,
        })
      }
    }
    setLoading(false)
  }

  const handleConfirm = async () => {
    if (!active_ring_id || !selectedTemplate) return
    setLoading(true)
    try {
      await api.post(`/rings/${active_ring_id}/blueprint/confirm`, {})
      
      if (preview) {
        for (const node of preview.nodes) {
          await createNode(active_ring_id, node.label, node.node_type)
        }
        
        useGraphStore.getState().fetchGraph(active_ring_id)
      }
      
      setConfirmed(true)
    } catch (e) {
      console.error('Failed to confirm blueprint:', e)
    }
    setLoading(false)
  }

  if (confirmed) {
    return (
      <div style={{ padding: 20, textAlign: 'center' }}>
        <div style={{ fontSize: 48, marginBottom: 16 }}>✓</div>
        <h2 style={{ color: 'var(--accent-green)', marginBottom: 8 }}>Blueprint Confirmed</h2>
        <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
          Your ring blueprint has been set up. You can now start building your knowledge graph.
        </p>
      </div>
    )
  }

  return (
    <div style={{ padding: '12px 16px', height: '100%', overflow: 'auto' }}>
      <h2 style={{ fontSize: 14, color: 'var(--accent-ice)', marginBottom: 12 }}>
        Choose Blueprint Template
      </h2>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 8, marginBottom: 16 }}>
        {TEMPLATES.map((template) => (
          <button
            key={template.id}
            onClick={() => handlePreview(template.id)}
            style={{
              padding: '10px 12px',
              background: selectedTemplate === template.id ? 'var(--bg-hover)' : 'var(--bg-input)',
              border: `1px solid ${selectedTemplate === template.id ? 'var(--accent-cyan)' : 'var(--border)'}`,
              borderRadius: 4,
              cursor: 'pointer',
              textAlign: 'left',
            }}
          >
            <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-primary)' }}>
              {template.name}
            </div>
            <div style={{ fontSize: 10, color: 'var(--text-dim)', marginTop: 2 }}>
              {template.description}
            </div>
            <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 4 }}>
              {template.nodes.length} nodes · {template.edges.length} edges
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
            padding: 12,
            marginBottom: 12,
          }}
        >
          <h3 style={{ fontSize: 11, color: 'var(--accent-ice)', marginBottom: 8 }}>Preview</h3>
          
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4 }}>Nodes:</div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
              {preview.nodes.map((node, i) => (
                <span
                  key={i}
                  style={{
                    fontSize: 9,
                    background: 'var(--bg-hover)',
                    padding: '2px 6px',
                    borderRadius: 2,
                    color: 'var(--text-secondary)',
                  }}
                >
                  {node.label}
                </span>
              ))}
            </div>
          </div>

          {preview.edges.length > 0 && (
            <div>
              <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4 }}>Edges:</div>
              <div style={{ fontSize: 9, color: 'var(--text-muted)', lineHeight: 1.8 }}>
                {preview.edges.map((edge, i) => (
                  <div key={i}>
                    {edge.from} <span style={{ color: 'var(--accent-cyan)' }}>→</span> {edge.to}
                    <span style={{ color: 'var(--text-dim)' }}> ({edge.relation})</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {selectedTemplate && (
        <button
          onClick={handleConfirm}
          disabled={loading}
          style={{
            width: '100%',
            padding: '8px 12px',
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            fontSize: 12,
            fontWeight: 700,
            cursor: loading ? 'default' : 'pointer',
            opacity: loading ? 0.6 : 1,
          }}
        >
          {loading ? 'Setting up...' : 'Confirm & Apply Blueprint'}
        </button>
      )}
    </div>
  )
}
