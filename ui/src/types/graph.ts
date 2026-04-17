export type NodeType = 'topic' | 'category' | 'leaf'
export type EdgeRelation = 'depends_on' | 'related_to' | 'derives_from' | 'contradicts'

export interface GraphNode {
  id: string
  label: string
  parent_id: string | null
  markdown_path: string
  node_type: NodeType
  tags: string[]
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface GraphEdge {
  id: string
  source_id: string
  target_id: string
  relation: EdgeRelation
  label: string
  created_at: string
}

export interface Graph {
  id: string
  name: string
  ring_id: string
  nodes: GraphNode[]
  edges: GraphEdge[]
  created_at: string
  updated_at: string
}
