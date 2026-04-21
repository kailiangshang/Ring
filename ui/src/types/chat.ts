export type MessageRole = 'user' | 'group_ring' | 'super_ring' | 'session_ring' | 'self' | 'system'

export interface ChatMessage {
  id: string
  role: MessageRole
  sender_name: string
  content: string
  node_refs?: string[]
  tag_refs?: string[]
  token_usage?: { prompt_tokens?: number; completion_tokens?: number; total_tokens?: number }
  created_at: string
}
