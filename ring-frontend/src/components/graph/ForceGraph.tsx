import { useEffect, useRef } from 'react'
import * as d3 from 'd3'
import type { GraphNode, GraphEdge } from '../../types'

const NODE_COLORS: Record<string, string> = {
  concept: '#4a90d9',
  category: '#e8913a',
  document: '#5cb85c',
  event: '#9b59b6',
  person: '#17a2b8',
  task: '#d9534f',
}

interface SimNode extends d3.SimulationNodeDatum {
  id: string
  label: string
  node_type: string
}

interface SimEdge extends d3.SimulationLinkDatum<SimNode> {
  id: string
  relation: string
}

interface ForceGraphProps {
  nodes: GraphNode[]
  edges: GraphEdge[]
  on_node_click: (node_id: string) => void
  selected_node_id: string | null
}

export function ForceGraph({ nodes, edges, on_node_click, selected_node_id }: ForceGraphProps) {
  const svg_ref = useRef<SVGSVGElement>(null)
  const container_ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!svg_ref.current || !container_ref.current) return

    const svg = d3.select(svg_ref.current)
    svg.selectAll('*').remove()

    const width = container_ref.current.clientWidth
    const height = container_ref.current.clientHeight

    svg.attr('width', width).attr('height', height)

    const sim_nodes: SimNode[] = nodes.map((n) => ({
      id: n.id,
      label: n.label,
      node_type: n.node_type,
    }))

    const node_map = new Map(sim_nodes.map((n) => [n.id, n]))

    const sim_edges: SimEdge[] = []
    for (const e of edges) {
      const source = node_map.get(e.source_id)
      const target = node_map.get(e.target_id)
      if (source && target) {
        sim_edges.push({ id: e.id, source, target, relation: e.relation })
      }
    }

    const g = svg.append('g')

    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform)
      })

    svg.call(zoom)

    const simulation = d3.forceSimulation<SimNode>(sim_nodes)
      .force(
        'link',
        d3.forceLink<SimNode, SimEdge>(sim_edges).id((d) => d.id),
      )
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2))

    const link = g
      .append('g')
      .selectAll('line')
      .data(sim_edges)
      .join('line')
      .attr('stroke', '#999')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 1.5)

    const node = g
      .append('g')
      .selectAll<SVGCircleElement, SimNode>('circle')
      .data(sim_nodes)
      .join('circle')
      .attr('r', 8)
      .attr('fill', (d) => NODE_COLORS[d.node_type] || '#888')
      .attr('cursor', 'pointer')
      .on('click', (_event, d) => {
        on_node_click(d.id)
      })
      .call(
        d3.drag<SVGCircleElement, SimNode>()
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
      .selectAll<SVGTextElement, SimNode>('text')
      .data(sim_nodes)
      .join('text')
      .text((d) => d.label)
      .attr('font-size', 10)
      .attr('dx', 12)
      .attr('dy', 4)

    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as SimNode).x!)
        .attr('y1', (d) => (d.source as SimNode).y!)
        .attr('x2', (d) => (d.target as SimNode).x!)
        .attr('y2', (d) => (d.target as SimNode).y!)

      node.attr('cx', (d) => d.x!).attr('cy', (d) => d.y!)
      label.attr('x', (d) => d.x!).attr('y', (d) => d.y!)
    })

    return () => {
      simulation.stop()
    }
  }, [nodes, edges])

  useEffect(() => {
    if (!svg_ref.current) return

    const svg = d3.select(svg_ref.current)
    const g = svg.select<SVGGElement>('g')

    g.selectAll<SVGCircleElement, SimNode>('circle')
      .data(
        nodes.map((n) => ({ id: n.id, label: n.label, node_type: n.node_type })),
        (d) => d.id,
      )
      .attr('stroke', (d) => (d.id === selected_node_id ? '#000' : 'none'))
      .attr('stroke-width', (d) => (d.id === selected_node_id ? 3 : 0))
  }, [selected_node_id, nodes])

  return (
    <div ref={container_ref} data-testid="graph-container" style={{ width: '100%', height: '100%' }}>
      <svg ref={svg_ref} />
    </div>
  )
}
