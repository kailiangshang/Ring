export type Role = 'creator' | 'admin' | 'member' | 'readonly'

export interface Ring {
  id: string
  name: string
  role: Role
  member_count: number
  node_count: number
  last_activity_at: string
  has_active_session: boolean
}

export interface Member {
  token_id: string
  display_name: string
  avatar: string | null
  role: Role
  joined_at: string
  online: boolean
}