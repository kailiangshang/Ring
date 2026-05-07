import { useEffect, useRef, useCallback } from 'react'
import * as d3 from 'd3'
import type { GraphNode, GraphEdge } from '../../types/graph'

interface SimNode extends d3.SimulationNodeDatum {
  id: string
  label: string
  node_type: string
}

interface SimEdge extends d3.SimulationLinkDatum<SimNode> {
  id: string
  relation: string
}

interface GraphCanvasProps {
  nodes: GraphNode[]
  edges: GraphEdge[]
  selectedNodeId: string | null
  collapsedNodes: Set<string>
  onSelectNode: (id: string | null) => void
  onToggleCollapse: (id: string) => void
  fullscreen?: boolean
}

const NODE_STYLES: Record<string, { fill: string; stroke: string; icon: string }> = {
  topic: { fill: '#0891B2', stroke: '#22d3ee', icon: '●' },
  category: { fill: '#16a34a', stroke: '#4ade80', icon: '◆' },
  leaf: { fill: '#d97706', stroke: '#fbbf24', icon: '▸' },
}

const EDGE_STYLES: Record<string, { stroke: string; dash: string }> = {
  related_to: { stroke: '#475569', dash: '' },
  depends_on: { stroke: '#6366f1', dash: '6,3' },
  derives_from: { stroke: '#8b5cf6', dash: '3,3' },
  contradicts: { stroke: '#ef4444', dash: '8,4' },
}

export function GraphCanvas({ nodes, edges, selectedNodeId, collapsedNodes, onSelectNode, onToggleCollapse, fullscreen }: GraphCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const simRef = useRef<d3.Simulation<SimNode, SimEdge> | null>(null)
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null)
  const gRef = useRef<d3.Selection<SVGGElement, unknown, null, undefined> | null>(null)
  const savedTransform = useRef<d3.ZoomTransform>(d3.zoomIdentity)

  const getVisibleNodes = useCallback(() => {
    const visibleIds = new Set<string>()
    nodes.forEach((n) => visibleIds.add(n.id))
    collapsedNodes.forEach((collapsedId) => {
      const children = edges
        .filter((e) => e.source_id === collapsedId)
        .map((e) => e.target_id)
      children.forEach((childId) => visibleIds.delete(childId))
    })
    return nodes.filter((n) => visibleIds.has(n.id))
  }, [nodes, edges, collapsedNodes])

  const getVisibleEdges = useCallback((visibleNodeIds: Set<string>) => {
    return edges.filter(
      (e) => visibleNodeIds.has(e.source_id) && visibleNodeIds.has(e.target_id)
    )
  }, [edges])

  const render = useCallback(() => {
    if (!svgRef.current || !containerRef.current) return

    const svg = d3.select(svgRef.current)
    const width = containerRef.current.clientWidth || 400
    const height = containerRef.current.clientHeight || 300

    const visibleNodes = getVisibleNodes()
    const visibleNodeIds = new Set(visibleNodes.map((n) => n.id))
    const visibleEdges = getVisibleEdges(visibleNodeIds)

    const simNodes: SimNode[] = visibleNodes.map((n) => ({
      id: n.id,
      label: n.label,
      node_type: n.node_type,
    }))

    const simEdges: SimEdge[] = visibleEdges.map((e) => ({
      source: e.source_id,
      target: e.target_id,
      id: e.id,
      relation: e.relation,
    }))

    let g = gRef.current
    let zoom = zoomRef.current

    if (!g) {
      svg.selectAll('*').remove()
      g = svg.append('g') as d3.Selection<SVGGElement, unknown, null, undefined>
      gRef.current = g

      zoom = d3
        .zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.2, 4])
        .on('zoom', (event) => {
          savedTransform.current = event.transform
          g!.attr('transform', event.transform as string)
        })

      svg.call(zoom)
      zoomRef.current = zoom
    }

    if (simRef.current) {
      simRef.current.stop()
    }

    const nodeR = fullscreen ? 20 : 14
    const labelFs = fullscreen ? 11 : 10
    const iconFs = fullscreen ? 14 : 10
    const linkDist = fullscreen ? 120 : 80

    let simulation: d3.Simulation<SimNode, SimEdge>
    try {
      simulation = d3
        .forceSimulation<SimNode>(simNodes)
        .force('link', d3.forceLink<SimNode, SimEdge>(simEdges).id((d) => d.id).distance(linkDist).strength(0.8))
        .force('charge', d3.forceManyBody<SimNode>().strength(-400).distanceMax(500))
        .force('center', d3.forceCenter(width / 2, height / 2))
        .force('collision', d3.forceCollide<SimNode>().radius(nodeR * 2.5))
        .force('x', d3.forceX(width / 2).strength(0.05))
        .force('y', d3.forceY(height / 2).strength(0.05))
        .alphaDecay(0.08)
        .velocityDecay(0.6)

      simRef.current = simulation
    } catch (err) {
      console.error('GraphCanvas: failed to create simulation', err, { simNodes, simEdges })
      return
    }

    const hasChildren = (nodeId: string) => {
      return edges.some((e) => e.source_id === nodeId)
    }

    g.selectAll<SVGDefsElement, unknown>('.arrow-defs').data([0]).join('defs').attr('class', 'arrow-defs')
      .selectAll('marker')
      .data(Object.entries(EDGE_STYLES))
      .join('marker')
      .attr('id', ([rel]) => `arrow-${rel}`)
      .attr('viewBox', '0 -4 8 8')
      .attr('refX', nodeR + 10)
      .attr('refY', 0)
      .attr('markerWidth', 5)
      .attr('markerHeight', 5)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-3L6,0L0,3')
      .attr('fill', ([, style]) => style.stroke)

    g.selectAll<SVGLineElement, SimEdge>('.graph-edge')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('line').attr('class', 'graph-edge')
          .attr('stroke', (d) => EDGE_STYLES[d.relation]?.stroke ?? '#475569')
          .attr('stroke-width', 1.2)
          .attr('stroke-opacity', 0.5)
          .attr('stroke-dasharray', (d) => EDGE_STYLES[d.relation]?.dash ?? '')
          .attr('marker-end', (d) => `url(#arrow-${d.relation})`),
        (update) => update
          .attr('stroke', (d) => EDGE_STYLES[d.relation]?.stroke ?? '#475569')
          .attr('stroke-dasharray', (d) => EDGE_STYLES[d.relation]?.dash ?? '')
          .attr('marker-end', (d) => `url(#arrow-${d.relation})`),
        (exit) => exit.remove(),
      )

    const link = g.selectAll<SVGLineElement, SimEdge>('.graph-edge')

    g.selectAll<SVGTextElement, SimEdge>('.edge-label')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('text').attr('class', 'edge-label')
          .attr('fill', (d) => EDGE_STYLES[d.relation]?.stroke ?? '#475569')
          .attr('font-size', fullscreen ? 9 : 7)
          .attr('text-anchor', 'middle')
          .attr('dy', -6)
          .attr('opacity', 0.7)
          .text((d) => d.relation.replace('_', ' ')),
        (update) => update
          .attr('fill', (d) => EDGE_STYLES[d.relation]?.stroke ?? '#475569'),
        (exit) => exit.remove(),
      )

    const linkLabel = g.selectAll<SVGTextElement, SimEdge>('.edge-label')

    g.selectAll<SVGGElement, SimNode>('.node-group')
      .data(simNodes, (d) => d.id)
      .join(
        (enter) => {
          const grp = enter.append('g').attr('class', 'node-group')

          grp.append('circle')
            .attr('class', 'node-shadow')
            .attr('r', nodeR + 3)
            .attr('fill', 'rgba(0,0,0,0.25)')
            .attr('pointer-events', 'none')

          grp.append('rect')
            .attr('class', 'node-body')
            .attr('x', -nodeR)
            .attr('y', -nodeR)
            .attr('width', nodeR * 2)
            .attr('height', nodeR * 2)
            .attr('rx', 6)
            .attr('ry', 6)
            .attr('fill', (d) => NODE_STYLES[d.node_type]?.fill ?? '#0891B2')
            .attr('stroke', (d) => NODE_STYLES[d.node_type]?.stroke ?? '#22d3ee')
            .attr('stroke-width', 1.5)
            .attr('cursor', 'pointer')
            .on('click', (_event, d) => {
              onSelectNode(d.id === selectedNodeId ? null : d.id)
            })
            .call(
              d3
                .drag<SVGRectElement, SimNode>()
                .on('start', (event, d) => {
                  if (!event.active) simulation.alphaTarget(0.15).restart()
                  d.fx = d.x
                  d.fy = d.y
                })
                .on('drag', (event, d) => {
                  d.fx = event.x
                  d.fy = event.y
                })
                .on('end', (event, d) => {
                  if (!event.active) simulation.alphaTarget(0)
                  d.fx = d.x
                  d.fy = d.y
                }),
            )

          grp.append('text')
            .attr('class', 'node-icon')
            .attr('dy', '0.35em')
            .attr('text-anchor', 'middle')
            .attr('fill', '#fff')
            .attr('font-size', iconFs)
            .attr('pointer-events', 'none')
            .text((d) => NODE_STYLES[d.node_type]?.icon ?? '●')

          grp.append('title').attr('class', 'node-tooltip')

          grp.append('text')
            .attr('class', 'node-label')
            .attr('dy', nodeR + 14)
            .attr('text-anchor', 'middle')
            .attr('fill', 'var(--text-secondary)')
            .attr('font-size', labelFs)
            .attr('font-family', 'inherit')
            .attr('pointer-events', 'none')

          grp.filter((d) => hasChildren(d.id))
            .append('g')
            .attr('class', 'collapse-btn')
            .attr('transform', `translate(${nodeR - 2},${-nodeR + 2})`)
            .on('click', (event, d) => {
              event.stopPropagation()
              onToggleCollapse(d.id)
            })
            .each(function () {
              d3.select(this).append('circle')
                .attr('r', 7)
                .attr('fill', 'var(--bg-panel)')
                .attr('stroke', 'var(--border)')
                .attr('stroke-width', 1)
                .attr('cursor', 'pointer')
              d3.select(this).append('text')
                .attr('class', 'collapse-text')
                .attr('dy', '0.35em')
                .attr('text-anchor', 'middle')
                .attr('fill', 'var(--text-primary)')
                .attr('font-size', 10)
                .attr('font-weight', 700)
                .attr('pointer-events', 'none')
            })

          return grp
        },
        (update) => update,
        (exit) => exit.remove(),
      )

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-body')
      .attr('stroke', (d) => NODE_STYLES[d.node_type]?.stroke ?? '#22d3ee')
      .attr('stroke-width', 1.5)

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-shadow')
      .attr('opacity', 0.25)

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-tooltip')
      .text((d) => `${d.label} [${d.node_type}]`)

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-label')
      .text((d) => d.label.length > (fullscreen ? 14 : 10) ? d.label.slice(0, fullscreen ? 14 : 10) + '\u2026' : d.label)

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.collapse-text')
      .text((d) => hasChildren(d.id) ? (collapsedNodes.has(d.id) ? '+' : '−') : '')

    const nodeGroup = g.selectAll<SVGGElement, SimNode>('.node-group')

    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as SimNode).x ?? 0)
        .attr('y1', (d) => (d.source as SimNode).y ?? 0)
        .attr('x2', (d) => (d.target as SimNode).x ?? 0)
        .attr('y2', (d) => (d.target as SimNode).y ?? 0)

      linkLabel
        .attr('x', (d) => (((d.source as SimNode).x ?? 0) + ((d.target as SimNode).x ?? 0)) / 2)
        .attr('y', (d) => (((d.source as SimNode).y ?? 0) + ((d.target as SimNode).y ?? 0)) / 2)

      nodeGroup.attr('transform', (d) => `translate(${d.x ?? 0},${d.y ?? 0})`)
    })

    svg.call(zoom!.transform, savedTransform.current)

    return () => {
      simulation.stop()
    }
  }, [nodes, edges, collapsedNodes, onSelectNode, onToggleCollapse, getVisibleNodes, getVisibleEdges, fullscreen])

  useEffect(() => {
    const g = gRef.current
    if (!g) return
    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-body')
      .attr('stroke', (d) => d.id === selectedNodeId ? '#67E8F9' : (NODE_STYLES[d.node_type]?.stroke ?? '#22d3ee'))
      .attr('stroke-width', (d) => d.id === selectedNodeId ? 3 : 1.5)
    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-shadow')
      .attr('opacity', (d) => d.id === selectedNodeId ? 1 : 0.25)
  }, [selectedNodeId])

  useEffect(() => {
    const cleanup = render()
    return () => cleanup?.()
  }, [render])

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
    bg.setAttribute('width', '100%')
    bg.setAttribute('height', '100%')
    bg.setAttribute('fill', '#0a0a0a')
    clone.insertBefore(bg, clone.firstChild)
    const serializer = new XMLSerializer()
    const svgString = serializer.serializeToString(clone)
    const img = new Image()
    img.onload = () => {
      const canvas = document.createElement('canvas')
      canvas.width = 2400
      canvas.height = 1600
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      ctx.drawImage(img, 0, 0)
      canvas.toBlob((blob) => {
        if (!blob) return
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = 'graph.png'
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)
        URL.revokeObjectURL(url)
      }, 'image/png')
    }
    img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgString)
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
    const serializer = new XMLSerializer()
    const svgString = serializer.serializeToString(clone)
    const blob = new Blob([svgString], { type: 'image/svg+xml;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'graph.svg'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [])

  const resetZoom = useCallback(() => {
    if (!svgRef.current || !zoomRef.current) return
    const svg = d3.select(svgRef.current)
    svg.call(zoomRef.current.transform, d3.zoomIdentity)
  }, [])

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', background: 'var(--bg-base)', position: 'relative' }}>
      <svg ref={svgRef} width="100%" height="100%" />
      <div style={{ position: 'absolute', top: 8, right: 8, display: 'flex', gap: 4, zIndex: 10 }}>
        <button onClick={() => {
          if (!svgRef.current || !zoomRef.current) return
          const svg = d3.select(svgRef.current)
          const t = savedTransform.current.translate(0, -40).scale(1.2)
          svg.call(zoomRef.current.transform, t)
        }} title="Zoom in" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 12, cursor: 'pointer' }}>
          +
        </button>
        <button onClick={() => {
          if (!svgRef.current || !zoomRef.current) return
          const svg = d3.select(svgRef.current)
          const t = savedTransform.current.translate(0, 40).scale(0.8)
          svg.call(zoomRef.current.transform, t)
        }} title="Zoom out" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 12, cursor: 'pointer' }}>
          −
        </button>
        <button onClick={resetZoom} title="Reset view" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 6px', fontSize: 10, cursor: 'pointer' }}>
          ⊡
        </button>
        <button onClick={exportSVG} title="Download SVG" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 8px', fontSize: 10, cursor: 'pointer' }}>
          SVG
        </button>
        <button onClick={exportPNG} title="Download PNG" style={{ background: 'var(--bg-hover)', color: 'var(--text-secondary)', border: '1px solid var(--border)', borderRadius: 3, padding: '2px 8px', fontSize: 10, cursor: 'pointer' }}>
          PNG
        </button>
      </div>
      <div style={{ position: 'absolute', bottom: 8, left: 8, display: 'flex', gap: 8, zIndex: 10 }}>
        {Object.entries(NODE_STYLES).map(([type, style]) => (
          <span key={type} style={{ fontSize: 9, color: 'var(--text-dim)', display: 'flex', alignItems: 'center', gap: 3 }}>
            <span style={{ width: 8, height: 8, borderRadius: 2, background: style.fill, display: 'inline-block' }} />
            {type}
          </span>
        ))}
        {Object.entries(EDGE_STYLES).map(([rel, style]) => (
          <span key={rel} style={{ fontSize: 9, color: style.stroke, display: 'flex', alignItems: 'center', gap: 3 }}>
            <span style={{ width: 14, height: 0, borderTop: `${style.dash ? '1px dashed' : '1px solid'} ${style.stroke}`, display: 'inline-block' }} />
            {rel.replace('_', ' ')}
          </span>
        ))}
      </div>
    </div>
  )
}
