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
}

const NODE_COLORS: Record<string, string> = {
  topic: '#0891B2',
  category: '#22c55e',
  leaf: '#f59e0b',
}

export function GraphCanvas({ nodes, edges, selectedNodeId, collapsedNodes, onSelectNode, onToggleCollapse }: GraphCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const simRef = useRef<d3.Simulation<SimNode, SimEdge> | null>(null)
  const zoomRef = useRef<d3.ZoomBehavior<SVGSVGElement, unknown> | null>(null)
  const gRef = useRef<d3.Selection<SVGGElement, unknown, null, undefined> | null>(null)
  const savedTransform = useRef<d3.ZoomTransform>(d3.zoomIdentity)

  // Filter nodes based on collapsed state
  const getVisibleNodes = useCallback(() => {
    const visibleIds = new Set<string>()
    
    // Start with all nodes, then hide children of collapsed nodes
    nodes.forEach((n) => visibleIds.add(n.id))
    
    collapsedNodes.forEach((collapsedId) => {
      // Find all children (targets of edges from this node)
      const children = edges
        .filter((e) => e.source_id === collapsedId)
        .map((e) => e.target_id)
      
      // Remove children from visible set
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
        .scaleExtent([0.3, 4])
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

    const simulation = d3
      .forceSimulation<SimNode>(simNodes)
      .force('link', d3.forceLink<SimNode, SimEdge>(simEdges).id((d) => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))

    simRef.current = simulation

    const hasChildren = (nodeId: string) => {
      return edges.some((e) => e.source_id === nodeId)
    }

    g.selectAll<SVGGElement, SimNode>('.node-group')
      .data(simNodes, (d) => d.id)
      .join(
        (enter) => {
          const grp = enter.append('g').attr('class', 'node-group')

          grp.append('circle')
            .attr('class', 'node-main')
            .attr('r', 12)
            .attr('fill', (d) => NODE_COLORS[d.node_type] ?? '#0891B2')
            .attr('cursor', 'pointer')
            .on('click', (_event, d) => {
              onSelectNode(d.id === selectedNodeId ? null : d.id)
            })
            .call(
              d3
                .drag<SVGCircleElement, SimNode>()
                .on('start', (event, d) => {
                  if (!event.active) simulation.alphaTarget(0.3).restart()
                  d.fx = d.x
                  d.fy = d.y
                })
                .on('drag', (event, d) => {
                  d.fx = event.x
                  d.fy = event.y
                })
                .on('end', (event, d) => {
                  if (!event.active) simulation.alphaTarget(0)
                  d.fx = null
                  d.fy = null
                }),
            )

          grp.append('title').attr('class', 'node-tooltip')

          grp.filter((d) => hasChildren(d.id))
            .append('circle')
            .attr('class', 'collapse-hit')
            .attr('r', 12)
            .attr('cx', 14)
            .attr('cy', -14)
            .attr('fill', 'transparent')
            .attr('cursor', 'pointer')
            .on('click', (event, d) => {
              event.stopPropagation()
              onToggleCollapse(d.id)
            })

          grp.filter((d) => hasChildren(d.id))
            .append('circle')
            .attr('class', 'collapse-bg')
            .attr('r', 6)
            .attr('cx', 14)
            .attr('cy', -14)
            .attr('fill', 'var(--bg-panel)')
            .attr('stroke', 'var(--border)')
            .attr('stroke-width', 1)
            .attr('pointer-events', 'none')

          grp.filter((d) => hasChildren(d.id))
            .append('text')
            .attr('class', 'collapse-icon')
            .attr('x', 14)
            .attr('y', -14)
            .attr('dy', '0.35em')
            .attr('text-anchor', 'middle')
            .attr('fill', 'var(--text-primary)')
            .attr('font-size', 10)
            .attr('font-weight', 700)
            .attr('pointer-events', 'none')

          grp.append('text')
            .attr('class', 'node-label')
            .attr('dy', 24)
            .attr('text-anchor', 'middle')
            .attr('fill', '#bfc7d5')
            .attr('font-size', 10)
            .attr('font-family', 'inherit')
            .attr('pointer-events', 'none')

          return grp
        },
        (update) => update,
        (exit) => exit.remove(),
      )

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-main')
      .attr('stroke', (d) => (d.id === selectedNodeId ? '#67E8F9' : 'none'))
      .attr('stroke-width', (d) => (d.id === selectedNodeId ? 3 : 0))

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-tooltip')
      .text((d) => `${d.label} [${d.node_type}]`)

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.node-label')
      .text((d) => (d.label.length > 12 ? d.label.slice(0, 12) + '\u2026' : d.label))

    g.selectAll<SVGGElement, SimNode>('.node-group').select('.collapse-icon')
      .text((d) => (hasChildren(d.id) ? (collapsedNodes.has(d.id) ? '+' : '-') : ''))

    const nodeGroup = g.selectAll<SVGGElement, SimNode>('.node-group')
    const label = g.selectAll<SVGTextElement, SimNode>('.node-label')

    g.selectAll<SVGLineElement, SimEdge>('.graph-edge')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('line').attr('class', 'graph-edge')
          .attr('stroke', '#1a2030')
          .attr('stroke-width', 1.5),
        (update) => update,
        (exit) => exit.remove(),
      )

    const link = g.selectAll<SVGLineElement, SimEdge>('.graph-edge')

    g.selectAll<SVGTextElement, SimEdge>('.edge-label')
      .data(simEdges, (d) => d.id)
      .join(
        (enter) => enter.append('text').attr('class', 'edge-label')
          .attr('fill', '#3a4550')
          .attr('font-size', 8)
          .attr('text-anchor', 'middle')
          .text((d) => d.relation),
        (update) => update,
        (exit) => exit.remove(),
      )

    const linkLabel = g.selectAll<SVGTextElement, SimEdge>('.edge-label')

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
      label.attr('x', (d) => d.x ?? 0).attr('y', (d) => d.y ?? 0)
    })

    svg.call(zoom!.transform, savedTransform.current)

    return () => {
      simulation.stop()
    }
  }, [nodes, edges, selectedNodeId, collapsedNodes, onSelectNode, onToggleCollapse, getVisibleNodes, getVisibleEdges])

  useEffect(() => {
    const cleanup = render()
    return () => cleanup?.()
  }, [render])

  const exportSVG = useCallback(() => {
    if (!svgRef.current) return
    const clone = svgRef.current.cloneNode(true) as SVGSVGElement
    clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
    clone.setAttribute('width', '1200')
    clone.setAttribute('height', '800')
    const style = document.createElementNS('http://www.w3.org/2000/svg', 'style')
    style.textContent = `text{font-family:system-ui,sans-serif}circle{transition:none}`
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
        <button
          onClick={() => {
            if (!svgRef.current || !zoomRef.current) return
            const svg = d3.select(svgRef.current)
            const t = savedTransform.current.translate(0, -40).scale(1.2)
            svg.call(zoomRef.current.transform, t)
          }}
          title="Zoom in"
          style={{
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 3,
            padding: '2px 6px',
            fontSize: 12,
            cursor: 'pointer',
          }}
        >
          +
        </button>
        <button
          onClick={() => {
            if (!svgRef.current || !zoomRef.current) return
            const svg = d3.select(svgRef.current)
            const t = savedTransform.current.translate(0, 40).scale(0.8)
            svg.call(zoomRef.current.transform, t)
          }}
          title="Zoom out"
          style={{
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 3,
            padding: '2px 6px',
            fontSize: 12,
            cursor: 'pointer',
          }}
        >
          −
        </button>
        <button
          onClick={resetZoom}
          title="Reset view"
          style={{
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 3,
            padding: '2px 6px',
            fontSize: 10,
            cursor: 'pointer',
          }}
        >
          ⊡
        </button>
        <button
          onClick={exportSVG}
          title="Download graph as SVG"
          style={{
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 3,
            padding: '2px 8px',
            fontSize: 10,
            cursor: 'pointer',
          }}
        >
          Download SVG
        </button>
      </div>
    </div>
  )
}
