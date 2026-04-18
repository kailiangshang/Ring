export interface ArchiveRecord {
  id: string
  ring_id: string
  session_id: string | null
  node_id: string | null
  file_name: string
  commit_sha: string | null
  branch: string | null
  merge_request_iid: number | null
  status: ArchiveStatus
  archived_by: string
  created_at: string
  updated_at: string
}

export type ArchiveStatus =
  | 'pending'
  | 'committed'
  | 'pushed'
  | 'mr_opened'
  | 'merged'
  | 'rejected'

export type NodeSuggestionAction =
  | 'create_new'
  | 'attach_existing'
  | 'update_existing'

export interface NodeSuggestion {
  action: NodeSuggestionAction
  parent_id?: string
  node_id?: string
  node_title?: string
}

export interface CreateArchiveInput {
  session_id?: string
  content: string
  suggested_title: string
  node_suggestion: NodeSuggestion
}

export interface ReviewInput {
  action: 'merge' | 'reject'
}

export interface ArchiveProgressEvent {
  step: string
  message: string
}

export interface RepoStatus {
  initialized: boolean
  has_remote: boolean
}
