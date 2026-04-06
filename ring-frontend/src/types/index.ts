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
