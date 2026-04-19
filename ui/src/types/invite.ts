export interface InviteToken {
  token: string
  ring_id: string
  type: 'open' | 'audit'
  role: string
  max_uses: number
  use_count: number
  max_members: number | null
  expires_at: string
  revoked_at: string | null
  created_by: string
  created_at: string
}

export interface JoinRequest {
  id: string
  ring_id: string
  invite_token: string
  display_name: string
  message: string | null
  status: 'pending' | 'approved' | 'rejected'
  reviewer_id: string | null
  review_note: string | null
  reviewed_at: string | null
  created_at: string
}

export interface CreateInviteInput {
  type: 'open' | 'audit'
  role?: string
  max_uses?: number
  max_members?: number | null
  expires_in_hours?: number
}

export interface JoinInfo {
  valid: boolean
  reason?: string | null
  ring_id?: string | null
  ring_name?: string | null
  member_count?: number | null
  role?: string | null
  token_type?: string | null
}
