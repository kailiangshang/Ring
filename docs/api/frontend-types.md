# 前端类型定义 API 参考

> 源码路径：`ring-frontend/src/types/index.ts`

## User

```typescript
interface User {
  user_id: string
  display_name: string
}
```

## SetupStatus

```typescript
interface SetupStatus {
  setup_completed: boolean
  step: string
  user_id?: string
}
```

## LlmConfig / GitlabConfig

```typescript
interface LlmConfig {
  provider: string
  model: string
  api_key: string
  base_url?: string | null
}

interface GitlabConfig {
  repo_url: string
  auth_type: string
  ssh_key_path?: string
  auto_create?: boolean
}
```

## Ring

```typescript
interface Ring {
  id: string
  name: string
  description?: string
  status: string
}

interface RingListItem {
  id: string
  name: string
  member_count: number
  graph_node_count: number
  last_activity_at: string
  role: string
}

interface CreateRingRequest {
  name: string
  description?: string
  role_description?: string
}
```

## Conversation / Message

```typescript
interface Conversation {
  id: string
  ring_id: string
  title: string
  context_mode: string
  created_at: string
}

interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant'
  content: string
  sender_id: string
  created_at: string
}
```

## SSE Event

```typescript
type SseEventType = 'text' | 'tool_call' | 'tool_result' | 'archive_suggestion' | 'blueprint_proposal' | 'done' | 'error'

interface SseEvent {
  type: SseEventType
  content?: string
  tool_name?: string
  tool_args?: Record<string, unknown>
  result?: unknown
  graphs?: GraphDef[]
  message?: string
  tool_call_id?: string
  tool?: string
  input?: unknown
  output?: unknown
  success?: boolean
  data?: unknown
}

interface ToolEvent {
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
```

## Blueprint

```typescript
interface GraphDef {
  name: string
  graph_type: string
  categories: string[]
}

interface BlueprintTemplate {
  id: string
  name: string
  description: string
  graphs: GraphDef[]
}

interface PreviewResponse {
  graphs: GraphDef[]
  preview: string
}

interface ConfirmResponse {
  success: boolean
  message: string
}
```

## Graph

```typescript
interface GraphNode {
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

interface GraphEdge {
  id: string
  source_id: string
  target_id: string
  relation: string
  label: string | null
  graph_id: string
}

interface GraphDetail {
  graph_id: string
  nodes: GraphNode[]
  edges: GraphEdge[]
}

interface NodeContent {
  node_id: string
  label: string
  markdown_path: string | null
  content: string | null
  last_modified: string
}

interface SearchResult {
  node_id: string
  graph_id: string
  label: string
  snippet: string
  rank: number
}
```

## Archive / Git

```typescript
interface ArchiveRequest {
  message_ids: string[]
  conversation_id: string
  graph_id: string
  target_node_id?: string
  label: string
}

interface ArchiveResponse {
  archive_id: string
  markdown_path: string
  git_status: string
  pr_url: string | null
  queue_position: number | null
}

interface ArchiveQueueResponse {
  current_review: QueueItem | null
  queue: QueueItem[]
}

interface QueueItem {
  pr_id: number
  author: string
  title: string
  position: number
}

interface PrListItem {
  pr_id: number
  title: string
  author: string
  state: string
}

interface PrDetail {
  pr_id: number
  title: string
  author: string
  changes: FileChange[]
}

interface FileChange {
  file: string
  status: string
  additions: number
  deletions: number
  diff: string
}

interface CommitLogEntry {
  id: string
  message: string
  author: string
  date: string
}
```

## Member / Invite

```typescript
interface InviteToken {
  token: string
  token_type: string
  role: string
  max_uses: number
  used_count: number
  created_at: string
}

interface Member {
  id: string
  ring_id: string
  user_id: string
  token_id: number
  display_name: string
  role: string
  joined_at: string
}
```

## Session

```typescript
interface SessionMessage {
  id: string
  session_id: string
  sender_id: string
  role: 'user' | 'assistant'
  content: string
  seq_num: number
  created_at: string
}

interface SessionMemberData {
  user_id: string
  role: string
  status: string
}

interface CreateSessionRequest {
  title?: string
  scenario: string
  archive_enabled?: boolean
  invite_member_ids?: string[]
}

interface InviteRequest {
  token_type: string
  role: string
  max_uses: number
  max_members?: number
}
```

## Settings

```typescript
interface Settings {
  llm_provider?: string
  llm_model?: string
  llm_api_key?: string
  llm_base_url?: string
  privacy_enabled?: string
}
```
