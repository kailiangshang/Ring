export type SessionPhase = 'material_prep' | 'discussion' | 'summary' | 'closed'
export type SessionSkill = 'decision' | 'research' | 'review' | 'retrospective' | 'knowledge_sharing' | 'discussion'

export interface Session {
  id: string
  ring_id: string
  title: string
  description: string
  skill: SessionSkill
  phase: SessionPhase
  owner: string
  archivable: boolean
  archive_enabled: boolean
  summary: string | null
  created_at: string
  updated_at: string
}

export interface SessionParticipant {
  session_id: string
  token_id: string
  role: 'owner' | 'participant'
  joined_at: string
}

export interface SessionMessage {
  id: string
  session_id: string
  seq_num: number
  sender: string
  sender_name: string
  content: string
  message_type: 'user' | 'system' | 'ai_delta' | 'ai_end'
  created_at: string
}

export interface CreateSessionInput {
  title: string
  description?: string
  skill: SessionSkill
  archivable?: boolean
  invitees?: string[]
}

export interface SessionDetail {
  session: Session
  participants: SessionParticipant[]
}

export interface SessionMaterial {
  id: string
  session_id: string
  item_type: 'document' | 'graph_node' | 'ai_generated'
  title: string
  content: string
  status: 'collecting' | 'analyzing' | 'ready'
  highlight: string | null
  created_at: string
}
