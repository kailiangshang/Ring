export type SessionPhase = 'material_prep' | 'discussion' | 'summary' | 'closed'
export type SessionSkill = 'decision' | 'research' | 'review' | 'retrospective' | 'knowledge_sharing' | 'discussion'

export interface Session {
  id: string
  title: string
  description: string
  skill: SessionSkill
  phase: SessionPhase
  owner: string
  participants: string[]
  archivable: boolean
  archive_enabled: boolean
  created_at: string
}
