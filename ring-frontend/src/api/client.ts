import type {
  SetupStatus,
  User,
  LlmConfig,
  GitlabConfig,
  Ring,
  RingListItem,
  CreateRingRequest,
  Conversation,
  Message,
  BlueprintTemplate,
  PreviewResponse,
  ConfirmResponse,
  GraphDetail,
  GraphNode,
  GraphEdge,
  NodeContent,
  SearchResult,
} from '../types'

const BASE_URL = '/api/v1'

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error(body.error || `request failed: ${res.status}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export async function get_setup_status(): Promise<SetupStatus> {
  return request<SetupStatus>('/setup/status')
}

export async function set_username(display_name: string): Promise<User> {
  return request<User>('/setup/username', {
    method: 'POST',
    body: JSON.stringify({ display_name }),
  })
}

export async function set_llm(config: LlmConfig): Promise<void> {
  return request<void>('/setup/llm', {
    method: 'POST',
    body: JSON.stringify(config),
  })
}

export async function set_gitlab(config: GitlabConfig): Promise<void> {
  return request<void>('/setup/gitlab', {
    method: 'POST',
    body: JSON.stringify(config),
  })
}

export async function complete_setup(): Promise<void> {
  return request<void>('/setup/complete', { method: 'POST' })
}

export async function list_rings(): Promise<RingListItem[]> {
  const data = await request<{ rings: RingListItem[] }>('/rings')
  return data.rings
}

export async function create_ring(req: CreateRingRequest): Promise<Ring> {
  return request<Ring>('/rings', {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function get_ring(id: string): Promise<Ring> {
  return request<Ring>(`/rings/${id}`)
}

export async function delete_ring(id: string): Promise<void> {
  return request<void>(`/rings/${id}`, { method: 'DELETE' })
}

export async function list_conversations(ring_id: string): Promise<Conversation[]> {
  const data = await request<{ conversations: Conversation[] }>(`/rings/${ring_id}/conversations`)
  return data.conversations
}

export async function create_conversation(
  ring_id: string,
  title: string,
): Promise<Conversation> {
  return request<Conversation>(`/rings/${ring_id}/conversations`, {
    method: 'POST',
    body: JSON.stringify({ title }),
  })
}

export async function get_messages(
  ring_id: string,
  conv_id: string,
): Promise<Message[]> {
  const data = await request<{ messages: Message[] }>(
    `/rings/${ring_id}/conversations/${conv_id}/messages`,
  )
  return data.messages
}

export function send_message(
  ring_id: string,
  conv_id: string,
  content: string,
): Promise<Response> {
  return fetch(`${BASE_URL}/rings/${ring_id}/conversations/${conv_id}/messages`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  })
}

export function super_ring_chat(message: string): Promise<Response> {
  return fetch(`${BASE_URL}/super-ring/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
  })
}

export async function list_blueprint_templates(
  ring_id: string,
): Promise<BlueprintTemplate[]> {
  const data = await request<{ templates: BlueprintTemplate[] }>(
    `/rings/${ring_id}/blueprint/templates`,
  )
  return data.templates
}

export function blueprint_chat(
  ring_id: string,
  message: string,
): Promise<Response> {
  return fetch(`${BASE_URL}/rings/${ring_id}/blueprint/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
  })
}

export async function blueprint_preview(
  ring_id: string,
  graphs: unknown[],
): Promise<PreviewResponse> {
  return request<PreviewResponse>(`/rings/${ring_id}/blueprint/preview`, {
    method: 'POST',
    body: JSON.stringify({ graphs }),
  })
}

export async function blueprint_confirm(
  ring_id: string,
  graphs: unknown[],
): Promise<ConfirmResponse> {
  return request<ConfirmResponse>(`/rings/${ring_id}/blueprint/confirm`, {
    method: 'POST',
    body: JSON.stringify({ graphs }),
  })
}

export async function list_graphs(ring_id: string): Promise<string[]> {
  return request<string[]>(`/rings/${ring_id}/graphs`)
}

export async function get_graph(ring_id: string, graph_id: string): Promise<GraphDetail> {
  return request<GraphDetail>(`/rings/${ring_id}/graphs/${graph_id}`)
}

export async function create_node(
  ring_id: string,
  graph_id: string,
  req: { label: string; node_type: string; parent_id?: string; description?: string },
): Promise<GraphNode> {
  return request<GraphNode>(`/rings/${ring_id}/graphs/${graph_id}/nodes`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function update_node(
  ring_id: string,
  graph_id: string,
  node_id: string,
  req: { label?: string; description?: string; node_type?: string },
): Promise<GraphNode> {
  return request<GraphNode>(`/rings/${ring_id}/graphs/${graph_id}/nodes/${node_id}`, {
    method: 'PUT',
    body: JSON.stringify(req),
  })
}

export async function delete_node(ring_id: string, graph_id: string, node_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/graphs/${graph_id}/nodes/${node_id}`, { method: 'DELETE' })
}

export async function get_node_content(ring_id: string, graph_id: string, node_id: string): Promise<NodeContent> {
  return request<NodeContent>(`/rings/${ring_id}/graphs/${graph_id}/nodes/${node_id}/content`)
}

export async function create_edge(
  ring_id: string,
  graph_id: string,
  req: { source_id: string; target_id: string; relation: string; label?: string },
): Promise<GraphEdge> {
  return request<GraphEdge>(`/rings/${ring_id}/graphs/${graph_id}/edges`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function delete_edge(ring_id: string, graph_id: string, edge_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/graphs/${graph_id}/edges/${edge_id}`, { method: 'DELETE' })
}

export async function search_nodes(
  ring_id: string,
  query: string,
  graph_ids?: string[],
): Promise<{ results: SearchResult[]; total: number }> {
  return request(`/rings/${ring_id}/search`, {
    method: 'POST',
    body: JSON.stringify({ query, graph_ids, limit: 20 }),
  })
}
