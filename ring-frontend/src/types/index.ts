export interface User {
  user_id: string
  display_name: string
}

export interface SetupStatus {
  setup_completed: boolean
  step: string
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
  description?: string
  status: string
}

export interface RingListItem {
  id: string
  name: string
  member_count: number
  graph_node_count: number
  last_activity_at: string
  role: string
}

export interface CreateRingRequest {
  name: string
  description?: string
  role_description?: string
}

export interface Conversation {
  id: string
  ring_id: string
  title: string
  context_mode: string
  created_at: string
}

export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant'
  content: string
  sender_id: string
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
  graphs?: GraphDef[]
  message?: string
}

export interface GraphDef {
  name: string
  graph_type: string
  categories: string[]
}

export interface BlueprintTemplate {
  id: string
  name: string
  description: string
  graphs: GraphDef[]
}

export interface PreviewResponse {
  graphs: GraphDef[]
  preview: string
}

export interface ConfirmResponse {
  success: boolean
  message: string
}
