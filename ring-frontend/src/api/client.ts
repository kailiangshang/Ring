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
  ArchiveRequest,
  ArchiveResponse,
  ArchiveQueueResponse,
  PrListItem,
  PrDetail,
  CommitLogEntry,
  Member,
  SessionData,
  CreateSessionRequest,
  InviteRequest,
  InviteToken,
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

export async function archive_content(
  ring_id: string,
  req: ArchiveRequest,
): Promise<ArchiveResponse> {
  return request<ArchiveResponse>(`/rings/${ring_id}/archive`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function get_archive_queue(
  ring_id: string,
): Promise<ArchiveQueueResponse> {
  return request<ArchiveQueueResponse>(`/rings/${ring_id}/archive/queue`)
}

export async function confirm_archive(
  ring_id: string,
  archive_id: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/archive/${archive_id}/confirm`, {
    method: 'POST',
  })
}

export async function list_prs(
  ring_id: string,
  state?: string,
): Promise<PrListItem[]> {
  const data = await request<{ prs: PrListItem[] }>(
    `/rings/${ring_id}/git/prs${state ? `?state=${state}` : ''}`,
  )
  return data.prs
}

export async function get_pr_diff(
  ring_id: string,
  pr_id: number,
): Promise<PrDetail> {
  return request<PrDetail>(`/rings/${ring_id}/git/prs/${pr_id}/diff`)
}

export async function merge_pr(
  ring_id: string,
  pr_id: number,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/git/prs/${pr_id}/merge`, {
    method: 'POST',
  })
}

export async function reject_pr(
  ring_id: string,
  pr_id: number,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/git/prs/${pr_id}/reject`, {
    method: 'POST',
  })
}

export async function get_commit_log(
  ring_id: string,
  limit?: number,
): Promise<CommitLogEntry[]> {
  const data = await request<{ commits: CommitLogEntry[] }>(
    `/rings/${ring_id}/git/commits${limit ? `?limit=${limit}` : ''}`,
  )
  return data.commits
}

export async function list_members(ring_id: string): Promise<Member[]> {
  const data = await request<{ members: Member[] }>(`/rings/${ring_id}/members`)
  return data.members
}

export async function generate_invite(
  ring_id: string,
  req: InviteRequest,
): Promise<InviteToken> {
  return request<InviteToken>(`/rings/${ring_id}/invites`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function update_member_role(
  ring_id: string,
  member_id: string,
  role: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/members/${member_id}/role`, {
    method: 'PUT',
    body: JSON.stringify({ role }),
  })
}

export async function remove_member(
  ring_id: string,
  member_id: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/members/${member_id}`, {
    method: 'DELETE',
  })
}

export async function join_ring(
  token: string,
  display_name: string,
): Promise<Member> {
  return request<Member>(`/rings/join?token=${token}`, {
    method: 'POST',
    body: JSON.stringify({ display_name }),
  })
}

export async function create_session(
  ring_id: string,
  req: CreateSessionRequest,
): Promise<SessionData> {
  return request<SessionData>(`/rings/${ring_id}/sessions`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function list_sessions(
  ring_id: string,
  status?: string,
): Promise<SessionData[]> {
  const data = await request<{ sessions: SessionData[] }>(
    `/rings/${ring_id}/sessions${status ? `?status=${status}` : ''}`,
  )
  return data.sessions
}

export async function get_session(
  ring_id: string,
  session_id: string,
): Promise<SessionData> {
  return request<SessionData>(`/rings/${ring_id}/sessions/${session_id}`)
}

export async function close_session(
  ring_id: string,
  session_id: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/close`, {
    method: 'POST',
  })
}

export async function leave_session(
  ring_id: string,
  session_id: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/leave`, {
    method: 'POST',
  })
}

export async function toggle_session_archive(
  ring_id: string,
  session_id: string,
  enabled: boolean,
): Promise<void> {
  return request<void>(
    `/rings/${ring_id}/sessions/${session_id}/archive-toggle`,
    {
      method: 'PUT',
      body: JSON.stringify({ archive_enabled: enabled }),
    },
  )
}

export async function invite_to_session(
  ring_id: string,
  session_id: string,
  member_ids: string[],
): Promise<void> {
  return request<void>(
    `/rings/${ring_id}/sessions/${session_id}/invite`,
    {
      method: 'POST',
      body: JSON.stringify({ member_ids }),
    },
  )
}

export async function delete_session(
  ring_id: string,
  session_id: string,
): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}`, {
    method: 'DELETE',
  })
}

export async function get_settings(): Promise<Record<string, string>> {
  return request<Record<string, string>>('/settings')
}

export async function update_settings(settings: Record<string, string>): Promise<void> {
  return request<void>('/settings', {
    method: 'PUT',
    body: JSON.stringify(settings),
  })
}
