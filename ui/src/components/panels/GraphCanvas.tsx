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
  onSelectNode: (id: string | null) => void
}

const NODE_COLORS: Record<string, string> = {
  topic: '#0891B2',
  category: '#22c55e',
  leaf: '#f59e0b',
}

export function GraphCanvas({ nodes, edges, selectedNodeId, onSelectNode }: GraphCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const render = useCallback(() => {
    if (!svgRef.current || !containerRef.current) return

    const svg = d3.select(svgRef.current)
    svg.selectAll('*').remove()

    const width = containerRef.current.clientWidth || 400
    const height = containerRef.current.clientHeight || 300

    const simNodes: SimNode[] = nodes.map((n) => ({
      id: n.id,
      label: n.label,
      node_type: n.node_type,
    }))

    const simEdges: SimEdge[] = edges.map((e) => ({
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

    const node = g
      .append('g')
      .selectAll<SVGCircleElement, SimNode>('circle')
      .data(simNodes)
      .join('circle')
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

      node.attr('cx', (d) => d.x ?? 0).attr('cy', (d) => d.y ?? 0)

      label.attr('x', (d) => d.x ?? 0).attr('y', (d) => d.y ?? 0)
    })

    return () => {
      simulation.stop()
    }
  }, [nodes, edges, selectedNodeId, onSelectNode])

  useEffect(() => {
    const cleanup = render()
    return () => cleanup?.()
  }, [render])

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', background: 'var(--bg-base)' }}>
      <svg ref={svgRef} width="100%" height="100%" />
    </div>
  )
}
