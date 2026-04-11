export interface User {
  user_id: string
  display_name: string
}

export interface SetupStatus {
  setup_completed: boolean
  step: string
  user_id?: string | null
}

export interface LlmConfig {
  provider: string
  model: string
  api_key: string
  base_url?: string | null
}

export interface GitlabConfig {
  repo_url: string
  auth_type: string
  ssh_key_path?: string
  auto_create?: boolean
}

export interface Ring {
  id: string
  name: string
  description: string | null
  creator_id: string
  gitlab_repo: string
  local_path: string
  next_token_id: number
  status: string
  created_at: string
  updated_at: string
}

export interface CreateRingRequest {
  name: string
  description?: string
  role_description?: string
}

export interface Conversation {
  id: string
  ring_id: string
  title: string | null
  mode: string
  context_mode: string
  token_count: number
  token_limit: number
  auto_compact: boolean
  summary: string | null
  compacted_at: string | null
  created_by: string
  created_at: string
  updated_at: string
}

export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant'
  content: string
  sender_id: string | null
  tool_calls: string | null
  archived: boolean
  created_at: string
}

export type SseEventType =
  | 'text'
  | 'tool_call'
  | 'tool_result'
  | 'archive_suggestion'
  | 'blueprint_proposal'
  | 'done'
  | 'error'

export interface SseEvent {
  type: SseEventType
  content?: string
  tool_name?: string
  tool_args?: Record<string, unknown>
  result?: unknown
  graphs?: GraphPreview[]
  message?: string
  tool_call_id?: string
  tool?: string
  input?: unknown
  output?: unknown
  success?: boolean
  data?: unknown
  code?: string
  message_id?: string | null
  token_usage?: TokenUsage | null
}

export interface TokenUsage {
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
}

export interface ToolEvent {
  id: string
  type: 'tool_call' | 'tool_result' | 'archive_suggestion'
  tool_call_id?: string
  tool_name?: string
  input?: unknown
  output?: unknown
  success?: boolean
  data?: unknown
  timestamp: number
}

export interface GraphDef {
  name: string
  graph_type: string
  categories: string[]
}

export interface BlueprintTemplate {
  id: string
  name: string
  description: string | null
  graphs: string
  is_system: boolean
  created_by: string | null
  created_at: string
}

export interface GraphPreviewNode {
  id: string
  label: string
  node_type: string
}

export interface GraphPreviewEdge {
  source_id: string
  target_id: string
  relation: string
}

export interface GraphPreview {
  name: string
  nodes: GraphPreviewNode[]
  edges: GraphPreviewEdge[]
}

export interface PreviewResponse {
  graphs: GraphPreview[]
}

export interface GraphInfo {
  id: string
  name: string
  graph_type: string
}

export interface ConfirmResponse {
  blueprint_id: string
  graphs: GraphInfo[]
  status: string
}

export interface GraphNode {
  id: string
  label: string
  node_type: string
  parent_id: string | null
  description: string | null
  graph_id: string
  markdown_path: string | null
  created_at: string
  updated_at: string
}

export interface GraphEdge {
  id: string
  source_id: string
  target_id: string
  relation: string
  label: string | null
  graph_id: string
}

export interface GraphDetail {
  graph_id: string
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface NodeContent {
  node_id: string
  label: string
  markdown_path: string | null
  content: string | null
  last_modified: string
}

export interface SearchResult {
  node_id: string
  graph_id: string
  label: string
  snippet: string
  rank: number
}

export interface ArchiveRequest {
  message_ids: string[]
  conversation_id: string
  graph_id: string
  target_node_id?: string
  label: string
}

export interface ArchiveResponse {
  archive_id: string
  markdown_path: string
  git_status: string
  pr_url: string | null
  queue_position: number | null
}

export interface ArchiveQueueResponse {
  current_review: QueueItem | null
  queue: QueueItem[]
}

export interface QueueItem {
  pr_id: number
  author: string
  title: string
  position: number
}

export interface PrListItem {
  pr_id: number
  title: string
  author: string
  state: string
  changes: FileChange[]
}

export interface FileChange {
  file: string
  status: string
  additions: number
  deletions: number
  diff: string
}

export interface CommitLogEntry {
  id: string
  message: string
  author: string
  date: string
}

export interface InviteToken {
  id: string
  ring_id: string
  token: string
  token_type: string
  role: string
  inviter_id: string
  max_uses: number
  use_count: number
  max_members: number | null
  expires_at: string
  used_at: string | null
  revoked_at: string | null
  created_at: string
}

export interface Member {
  id: string
  ring_id: string
  user_id: string
  token_id: number
  display_name: string
  role: string
  joined_at: string
}

export interface SessionMessage {
  id: string
  session_id: string
  sender_id: string
  role: 'user' | 'assistant'
  content: string
  seq_num: number
  created_at: string
}

export interface SessionMemberBrief {
  user_id: string
  role: string
  status: string
}

export interface SessionListItem {
  id: string
  title: string | null
  created_by: string
  member_count: number
  archive_enabled: boolean
  status: string
  created_at: string
}

export interface SessionDetail {
  id: string
  ring_id: string
  title: string | null
  scenario: string
  created_by: string
  archive_enabled: boolean
  status: string
  members: SessionMemberBrief[]
  created_at: string
}

export interface CreateSessionRequest {
  title?: string
  scenario: string
  archive_enabled?: boolean
  invite_member_ids?: string[]
}

export interface InviteRequest {
  token_type: string
  role: string
  max_uses: number
  max_members?: number
}

export interface Notification {
  id: string
  ring_id: string
  user_id: string
  type: string
  title: string
  body: string | null
  related_id: string | null
  is_read: boolean
  created_at: string
}

export type Settings = Record<string, string>
