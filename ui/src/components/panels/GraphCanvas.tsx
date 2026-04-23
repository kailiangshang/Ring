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
    svg.selectAll('*').remove()

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

    const g = svg.append('g')

    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.3, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform)
      })

    svg.call(zoom)

    const simulation = d3
      .forceSimulation<SimNode>(simNodes)
      .force('link', d3.forceLink<SimNode, SimEdge>(simEdges).id((d) => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))

    simRef.current = simulation

    const link = g
      .append('g')
      .selectAll('line')
      .data(simEdges)
      .join('line')
      .attr('stroke', '#1a2030')
      .attr('stroke-width', 1.5)

    const linkLabel = g
      .append('g')
      .selectAll('text')
      .data(simEdges)
      .join('text')
      .attr('fill', '#3a4550')
      .attr('font-size', 8)
      .attr('text-anchor', 'middle')
      .text((d) => d.relation)

    const nodeGroup = g
      .append('g')
      .selectAll('g')
      .data(simNodes)
      .join('g')

    // Main node circle
    nodeGroup
      .append('circle')
      .attr('r', 12)
      .attr('fill', (d) => NODE_COLORS[d.node_type] ?? '#0891B2')
      .attr('stroke', (d) => (d.id === selectedNodeId ? '#67E8F9' : 'none'))
      .attr('stroke-width', (d) => (d.id === selectedNodeId ? 3 : 0))
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

    // Collapse/expand button
    const hasChildren = (nodeId: string) => {
      return edges.some((e) => e.source_id === nodeId)
    }

    nodeGroup
      .filter((d) => hasChildren(d.id))
      .append('circle')
      .attr('r', 6)
      .attr('cx', 14)
      .attr('cy', -14)
      .attr('fill', 'var(--bg-panel)')
      .attr('stroke', 'var(--border)')
      .attr('stroke-width', 1)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation()
        onToggleCollapse(d.id)
      })

    nodeGroup
      .filter((d) => hasChildren(d.id))
      .append('text')
      .attr('x', 14)
      .attr('y', -14)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('fill', 'var(--text-primary)')
      .attr('font-size', 10)
      .attr('font-weight', 700)
      .attr('cursor', 'pointer')
      .attr('pointer-events', 'none')
      .text((d) => collapsedNodes.has(d.id) ? '+' : '-')

    const label = g
      .append('g')
      .selectAll('text')
      .data(simNodes)
      .join('text')
      .attr('dy', 24)
      .attr('text-anchor', 'middle')
      .attr('fill', '#bfc7d5')
      .attr('font-size', 10)
      .attr('font-family', 'inherit')
      .attr('pointer-events', 'none')
      .text((d) => (d.label.length > 12 ? d.label.slice(0, 12) + '…' : d.label))

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

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', background: 'var(--bg-base)', position: 'relative' }}>
      <svg ref={svgRef} width="100%" height="100%" />
      <button
        onClick={exportSVG}
        style={{
          position: 'absolute',
          top: 8,
          right: 8,
          background: 'var(--bg-hover)',
          color: 'var(--text-secondary)',
          border: '1px solid var(--border)',
          borderRadius: 3,
          padding: '2px 8px',
          fontSize: 10,
          cursor: 'pointer',
          zIndex: 10,
        }}
      >
        SVG
      </button>
    </div>
  )
}
