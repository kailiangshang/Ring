import { useEffect, useRef, useCallback } from 'react'
import * as d3 from 'd3'
import type { GraphNode, GraphEdge } from '../../types/graph'

interface SimNode {
  id: string
  label: string
  node_type: string
  x: number
  y: number
}

interface SimEdge {
  source: string
  target: string
  id: string
  relation: string
  label: string
}

interface GraphCanvasProps {
  nodes: GraphNode[]
  edges: GraphEdge[]
  selectedNodeId: string | null
  selectedEdgeId?: string | null
  relationDraftSourceId?: string | null
  relationDraftTargetId?: string | null
  collapsedNodes: Set<string>
  onSelectNode: (id: string | null) => void
  onSelectEdge?: (id: string | null) => void
  onToggleCollapse: (id: string) => void
  fullscreen?: boolean
}

const NODE_H = 26
const NODE_RX = 4
const NODE_PAD_X = 10
const NODE_PAD_LEFT = 20

const NODE_COLORS: Record<string, { bg: string; border: string; text: string }> = {
  topic: { bg: '#0e2a3a', border: '#22d3ee', text: '#e2e8f0' },
  category: { bg: '#0e2e1a', border: '#4ade80', text: '#e2e8f0' },
  leaf: { bg: '#2a1e0a', border: '#fbbf24', text: '#e2e8f0' },
}

const EDGE_COLORS: Record<string, string> = {
  related_to: '#475569',
  depends_on: '#6366f1',
  derives_from: '#8b5cf6',
  contradicts: '#ef4444',
}

const ctx = typeof document !== 'undefined' ? document.createElement('canvas').getContext('2d') : null

function measureText(text: string, fontSize: number): number {
  if (!ctx) return text.length * fontSize * 0.6
  ctx.font = `${fontSize}px system-ui, -apple-system, sans-serif`
  return ctx.measureText(text).width
}

export function GraphCanvas({
  nodes,
  edges,
  selectedNodeId,
  selectedEdgeId = null,
  relationDraftSourceId = null,
  relationDraftTargetId = null,
  collapsedNodes,
  onSelectNode,
  onSelectEdge,
  onToggleCollapse,
  fullscreen,
}: GraphCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null)
  const gRef = useRef<d3.Selection<SVGGElement, unknown, null, undefined> | null>(null)
  const savedTransform = useRef<d3.ZoomTransform>(d3.zoomIdentity)
  const posRef = useRef<Map<string, { x: number; y: number }>>(new Map())
  const widthRef = useRef<Map<string, number>>(new Map())
  const dataKeyRef = useRef('')
  const onSelectNodeRef = useRef(onSelectNode)
  const onToggleCollapseRef = useRef(onToggleCollapse)
  const selectedNodeIdRef = useRef(selectedNodeId)
  useEffect(() => {
    onSelectNodeRef.current = onSelectNode
    onToggleCollapseRef.current = onToggleCollapse
    selectedNodeIdRef.current = selectedNodeId
  })

  const getNodeBorder = useCallback((d: SimNode) => {
    if (d.id === relationDraftSourceId) return '#67E8F9'
    if (d.id === relationDraftTargetId) return '#F59E0B'
    if (d.id === selectedNodeId) return (NODE_COLORS[d.node_type] ?? NODE_COLORS.topic).border
    return 'var(--border)'
  }, [relationDraftSourceId, relationDraftTargetId, selectedNodeId])

  const getNodeBorderWidth = useCallback((d: SimNode) => {
    if (d.id === relationDraftSourceId || d.id === relationDraftTargetId) return 3
    if (d.id === selectedNodeId) return 2
    return 1
  }, [relationDraftSourceId, relationDraftTargetId, selectedNodeId])

  const getVisible = useCallback(() => {
    const visibleIds = new Set<string>()
    nodes.forEach((n) => visibleIds.add(n.id))
    collapsedNodes.forEach((cid) => {
      edges.filter((e) => e.source_id === cid).forEach((e) => visibleIds.delete(e.target_id))
    })
    const visNodes = nodes.filter((n) => visibleIds.has(n.id))
    const visEdges = edges.filter((e) => visibleIds.has(e.source_id) && visibleIds.has(e.target_id))
    return { visNodes, visEdges }
  }, [nodes, edges, collapsedNodes])

  const computeLayout = useCallback((visNodes: GraphNode[], visEdges: GraphEdge[], width: number, height: number): SimNode[] => {
    const fontSize = fullscreen ? 12 : 11
    const maxLabel = fullscreen ? 20 : 16

    const dataKey = visNodes.map((n) => n.id).join(',') + '|' + visEdges.map((e) => e.id).join(',')
    if (dataKey === dataKeyRef.current) {
      return visNodes.map((n) => {
        const cached = posRef.current.get(n.id)
        const lbl = n.label.length > maxLabel ? n.label.slice(0, maxLabel) + '\u2026' : n.label
        const w = measureText(lbl, fontSize) + NODE_PAD_LEFT + NODE_PAD_X
        widthRef.current.set(n.id, w)
        return { id: n.id, label: lbl, node_type: n.node_type, x: cached?.x ?? 0, y: cached?.y ?? 0 }
      })
    }
    dataKeyRef.current = dataKey

    const simNodes: d3.SimulationNodeDatum[] = visNodes.map((n) => {
      const cached = posRef.current.get(n.id)
      const lbl = n.label.length > maxLabel ? n.label.slice(0, maxLabel) + '\u2026' : n.label
      const w = measureText(lbl, fontSize) + NODE_PAD_LEFT + NODE_PAD_X
      widthRef.current.set(n.id, w)
      return { id: n.id, x: cached?.x, y: cached?.y }
    })

    const simLinks = visEdges.map((e) => ({ source: e.source_id, target: e.target_id }))

    const sim = d3.forceSimulation(simNodes)
      .force('link', d3.forceLink(simLinks).id((d: d3.SimulationNodeDatum) => (d as { id?: string }).id ?? '').distance(100).strength(0.5))
      .force('charge', d3.forceManyBody().strength(-300).distanceMax(400))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(50))
      .alphaDecay(0.1)
      .stop()

    for (let i = 0; i < 200; i++) sim.tick()
    sim.stop()

    type SimDatum = d3.SimulationNodeDatum & { id?: string }
    const pos = simNodes as SimDatum[]
    const result: SimNode[] = visNodes.map((n, i) => {
      const x = pos[i].x ?? width / 2
      const y = pos[i].y ?? height / 2
      posRef.current.set(n.id, { x, y })
      return { id: n.id, label: n.label.length > maxLabel ? n.label.slice(0, maxLabel) + '\u2026' : n.label, node_type: n.node_type, x, y }
    })
    return result
  }, [fullscreen])

  const render = useCallback(() => {
    if (!svgRef.current || !containerRef.current) return
    const g = gRef.current
    if (!g) return

    const { visNodes, visEdges } = getVisible()
    const width = containerRef.current.clientWidth || 400
    const height = containerRef.current.clientHeight || 300
    const simNodes = computeLayout(visNodes, visEdges, width, height)
    const fontSize = fullscreen ? 12 : 11

    const simEdges: SimEdge[] = visEdges.map((e) => ({
      source: e.source_id, target: e.target_id, id: e.id, relation: e.relation, label: e.label,
    }))

    const nodeMap = new Map(simNodes.map((n) => [n.id, n]))
    const hasChildren = (id: string) => edges.some((e) => e.source_id === id)

    const linkSel = g.selectAll<SVGLineElement, SimEdge>('.graph-edge')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('line').attr('class', 'graph-edge')
          .attr('stroke-width', 1),
        (update) => update,
        (exit) => exit.remove(),
      )
      .attr('stroke', (d) => EDGE_COLORS[d.relation] ?? '#475569')
      .attr('stroke-width', (d) => d.id === selectedEdgeId ? 2.6 : 1)
      .attr('stroke-opacity', (d) => {
        if (d.id === selectedEdgeId) return 0.95
        if (!selectedNodeIdRef.current) return 0.35
        return 0.1
      })
      .attr('marker-end', (d) => `url(#arrow-${d.relation})`)

    linkSel.each(function (d) {
      const s = endpointId(d.source)
      const t = endpointId(d.target)
      const sel = selectedNodeIdRef.current
      if (sel && (s === sel || t === sel)) {
        d3.select(this).attr('stroke-opacity', 0.8)
      }
    })

    g.selectAll<SVGLineElement, SimEdge>('.graph-edge-hit')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('line')
          .attr('class', 'graph-edge-hit')
          .attr('stroke', 'transparent')
          .attr('stroke-width', fullscreen ? 18 : 14)
          .attr('cursor', 'pointer')
          .on('click', (_event, d) => {
            onSelectEdge?.(d.id === selectedEdgeId ? null : d.id)
          }),
        (update) => update,
        (exit) => exit.remove(),
      )

    g.selectAll<SVGTextElement, SimEdge>('.edge-label')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('text').attr('class', 'edge-label')
          .attr('font-size', 8)
          .attr('text-anchor', 'middle')
          .attr('dy', -4)
          .attr('opacity', 0.55),
        (update) => update,
        (exit) => exit.remove(),
      )
      .attr('fill', (d) => EDGE_COLORS[d.relation] ?? '#475569')
      .text((d) => d.label?.trim() || d.relation.replace('_', ' '))

    const linkLabelSel = g.selectAll<SVGTextElement, SimEdge>('.edge-label')

    const nodeSel = g.selectAll<SVGGElement, SimNode>('.node-group')
      .data(simNodes, (d) => d.id)
      .join(
        (enter) => {
          const grp = enter.append('g').attr('class', 'node-group')

          grp.append('rect')
            .attr('class', 'node-body')
            .attr('height', NODE_H)
            .attr('rx', NODE_RX)
            .attr('ry', NODE_RX)
            .attr('cursor', 'pointer')

          grp.append('text')
            .attr('class', 'node-label')
            .attr('y', NODE_H / 2)
            .attr('dy', '0.35em')
            .attr('text-anchor', 'middle')
            .attr('font-size', fontSize)
            .attr('font-family', 'system-ui, -apple-system, sans-serif')
            .attr('pointer-events', 'none')

          grp.filter((d) => hasChildren(d.id))
            .append('g')
            .attr('class', 'collapse-btn')
            .each(function () {
              d3.select(this).append('rect')
                .attr('width', 14).attr('height', 14)
                .attr('rx', 2).attr('y', -NODE_H / 2 - 16)
                .attr('fill', 'var(--bg-panel)').attr('stroke', 'var(--border)').attr('stroke-width', 1)
                .attr('cursor', 'pointer')
              d3.select(this).append('text')
                .attr('class', 'collapse-text')
                .attr('x', 7).attr('y', -NODE_H / 2 - 9)
                .attr('text-anchor', 'middle')
                .attr('fill', 'var(--text-primary)').attr('font-size', 9).attr('font-weight', 700)
                .attr('pointer-events', 'none')
            })

          return grp
        },
        (update) => update,
        (exit) => exit.remove(),
      )

    nodeSel.select('.node-body')
      .attr('x', (d) => -(widthRef.current.get(d.id) ?? 80) / 2)
      .attr('y', 0)
      .attr('width', (d) => widthRef.current.get(d.id) ?? 80)
      .attr('fill', (d) => (NODE_COLORS[d.node_type] ?? NODE_COLORS.topic).bg)
      .attr('stroke', (d) => getNodeBorder(d))
      .attr('stroke-width', (d) => getNodeBorderWidth(d))

    nodeSel
      .call(d3.drag<SVGGElement, SimNode>()
        .on('start', (event) => {
          event.sourceEvent.stopPropagation()
          const d = event.subject as SimNode
          ;(d as SimNode & { __drag_dist?: number }).__drag_dist = 0
        })
        .on('drag', (event: d3.D3DragEvent<SVGGElement, SimNode, SimNode>, d) => {
          const scale = savedTransform.current.k
          d.x += event.dx / scale
          d.y += event.dy / scale
          posRef.current.set(d.id, { x: d.x, y: d.y })
          const dragged = d as SimNode & { __drag_dist?: number }
          dragged.__drag_dist = (dragged.__drag_dist ?? 0) + Math.abs(event.dx) + Math.abs(event.dy)
          nodeSel.filter((n) => n.id === d.id)
            .attr('transform', `translate(${d.x},${d.y})`)
          updateLinkPositions(linkSel, linkLabelSel, nodeMap)
        }),
      )

    nodeSel.select('.node-body')
      .on('click', (_event: MouseEvent, d: SimNode) => {
        _event.stopPropagation()
        const dragged = d as SimNode & { __drag_dist?: number }
        if ((dragged.__drag_dist ?? 0) > 4) {
          dragged.__drag_dist = 0
          return
        }
        dragged.__drag_dist = 0
        onSelectNodeRef.current(d.id === selectedNodeIdRef.current ? null : d.id)
      })

    nodeSel.select('.node-label')
      .attr('fill', (d) => (NODE_COLORS[d.node_type] ?? NODE_COLORS.topic).text)
      .text((d) => d.label)

    nodeSel.select('.collapse-btn')
      .attr('transform', (d) => `translate(${(widthRef.current.get(d.id) ?? 80) / 2 - 16}, 0)`)
      .on('click', (event, d) => {
        event.stopPropagation()
        onToggleCollapseRef.current(d.id)
      })
    nodeSel.select('.collapse-text')
      .text((d) => hasChildren(d.id) ? (collapsedNodes.has(d.id) ? '+' : '\u2212') : '')

    nodeSel.attr('transform', (d) => `translate(${d.x},${d.y})`)

    updateLinkPositions(linkSel, linkLabelSel, nodeMap)
  }, [getVisible, computeLayout, edges, collapsedNodes, fullscreen, selectedNodeId, selectedEdgeId, onSelectEdge, getNodeBorder, getNodeBorderWidth])

  useEffect(() => {
    if (!svgRef.current || !containerRef.current) return
    const svg = d3.select(svgRef.current)

    if (!gRef.current) {
      svg.selectAll('*').remove()

      const defs = svg.append('defs')
      Object.entries(EDGE_COLORS).forEach(([rel, color]) => {
        const marker = defs.append('marker')
          .attr('id', `arrow-${rel}`)
          .attr('viewBox', '0 -4 8 8')
          .attr('refX', NODE_H / 2 + 6)
          .attr('refY', 0)
          .attr('markerWidth', 4).attr('markerHeight', 4)
          .attr('orient', 'auto')
        marker.append('path').attr('d', 'M0,-3L6,0L0,3').attr('fill', color)
      })

      svg.on('click', () => {
        onSelectNodeRef.current(null)
      })

      const g = svg.append('g') as d3.Selection<SVGGElement, unknown, null, undefined>
      gRef.current = g

      const zoom = d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.2, 4])
        .on('zoom', (event) => {
          savedTransform.current = event.transform
          gRef.current?.attr('transform', event.transform as string)
        })
      svg.call(zoom)
      zoomRef.current = zoom
    }
  }, [])

  useEffect(() => {
    render()
  }, [render])

  const resetZoom = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return
    d3.select(svgRef.current).call(zoomRef.current.transform, d3.zoomIdentity)
  }, [])

  const exportSVG = useCallback(() => {
    if (!svgRef.current) return
    const clone = svgRef.current.cloneNode(true) as SVGSVGElement
    clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
    clone.setAttribute('width', '1200')
    clone.setAttribute('height', '800')
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style')
    style.textContent = `text{font-family:system-ui,sans-serif}rect{transition:none}`
    clone.insertBefore(style, clone.firstChild)
    const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect')
    bg.setAttribute('width', '100%')
    bg.setAttribute('height', '100%')
    bg.setAttribute('fill', '#0a0a0a')
    clone.insertBefore(bg, clone.firstChild)
    const serializer = new XMLSerializer()
    const svgString = serializer.serializeToString(clone)
    const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = 'graph.svg'
    document.body.appendChild(a); a.click(); document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [])

  const exportPNG = useCallback(() => {
    if (!svgRef.current) return
    const clone = svgRef.current.cloneNode(true) as SVGSVGElement
    clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
    clone.setAttribute('width', '2400')
    clone.setAttribute('height', '1600')
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style')
    style.textContent = `text{font-family:system-ui,sans-serif;fill:#e0e0e0}rect{transition:none}line{stroke:#444}`
    clone.insertBefore(style, clone.firstChild)
    const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect')
    bg.setAttribute('width', '100%'); bg.setAttribute('height', '100%'); bg.setAttribute('fill', '#0a0a0a')
    clone.insertBefore(bg, clone.firstChild)
    const serializer = new XMLSerializer()
    const svgString = serializer.serializeToString(clone)
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      canvas.width = 2400; canvas.height = 1600
      const c = canvas.getContext('2d')
      if (!c) return
      c.drawImage(img, 0, 0)
      canvas.toBlob((blob) => {
        if (!blob) return
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url; a.download = 'graph.png'
        document.body.appendChild(a); a.click(); document.body.removeChild(a)
        URL.revokeObjectURL(url)
      }, 'image/png')
    }
    img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgString)
  }, [])

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', background: 'var(--bg-base)', position: 'relative' }}>
      <svg ref={svgRef} width="100%" height="100%" />
      <div style={{ position: 'absolute', top: 8, right: 8, display: 'flex', gap: 4, zIndex: 10 }}>
        <button onClick={() => {
          if (!svgRef.current || !zoomRef.current) return
          d3.select(svgRef.current).call(zoomRef.current.transform, savedTransform.current.translate(0, -40).scale(1.2))
        }} title="Zoom in" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 12, cursor: 'pointer' }}>+</button>
        <button onClick={() => {
          if (!svgRef.current || !zoomRef.current) return
          d3.select(svgRef.current).call(zoomRef.current.transform, savedTransform.current.translate(0, 40).scale(0.8))
        }} title="Zoom out" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 12, cursor: 'pointer' }}>&minus;</button>
        <button onClick={resetZoom} title="Reset view" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 10, cursor: 'pointer' }}>&#8857;</button>
        <button onClick={exportSVG} title="SVG" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 8px', fontSize: 10, cursor: 'pointer' }}>SVG</button>
        <button onClick={exportPNG} title="PNG" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 8px', fontSize: 10, cursor: 'pointer' }}>PNG</button>
      </div>
      <div style={{ position: 'absolute', bottom: 8, left: 8, display: 'flex', gap: 8, zIndex: 10 }}>
        {Object.entries(NODE_COLORS).map(([type, c]) => (
          <span key={type} style={{ fontSize: 9, color: 'var(--text-dim)', display: 'flex', alignItems: 'center', gap: 3 }}>
            <span style={{ width: 12, height: 8, borderRadius: 2, background: c.bg, border: `1px solid ${c.border}`, display: 'inline-block' }} />
            {type}
          </span>
        ))}
        {Object.entries(EDGE_COLORS).map(([rel, c]) => (
          <span key={rel} style={{ fontSize: 9, color: c, display: 'flex', alignItems: 'center', gap: 3 }}>
            <span style={{ width: 12, height: 0, borderTop: `1px solid ${c}`, display: 'inline-block' }} />
            {rel.replace('_', ' ')}
          </span>
        ))}
      </div>
    </div>
  )
}

function endpointId(v: string | d3.SimulationNodeDatum): string {
  return typeof v === 'string' ? v : (v as { id?: string }).id ?? ''
}

function updateLinkPositions(
  linkSel: d3.Selection<SVGLineElement, SimEdge, SVGGElement, unknown>,
  linkLabelSel: d3.Selection<SVGTextElement, SimEdge, SVGGElement, unknown>,
  nodeMap: Map<string, SimNode>,
) {
  linkSel
    .attr('x1', (d) => (nodeMap.get(endpointId(d.source))?.x ?? 0))
    .attr('y1', (d) => (nodeMap.get(endpointId(d.source))?.y ?? 0) + NODE_H / 2)
    .attr('x2', (d) => (nodeMap.get(endpointId(d.target))?.x ?? 0))
    .attr('y2', (d) => (nodeMap.get(endpointId(d.target))?.y ?? 0) + NODE_H / 2)
  linkLabelSel
    .attr('x', (d) => {
      const s = nodeMap.get(endpointId(d.source))
      const t = nodeMap.get(endpointId(d.target))
      return ((s?.x ?? 0) + (t?.x ?? 0)) / 2
    })
    .attr('y', (d) => {
      const s = nodeMap.get(endpointId(d.source))
      const t = nodeMap.get(endpointId(d.target))
      return ((s?.y ?? 0) + (t?.y ?? 0)) / 2 + NODE_H / 2
    })
}
