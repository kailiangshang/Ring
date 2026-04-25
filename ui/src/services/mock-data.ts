import type { Ring, Member } from '../types/ring'
import type { ChatMessage } from '../types/chat'
import type { Session } from '../types/session'

export const MOCK_RINGS: Ring[] = [
  {
    id: '01JTYRING1',
    name: '竞品分析组',
    role: 'creator',
    storage_mode: 'local',
    member_count: 5,
    node_count: 13,
    last_activity_at: '2026-04-17T08:00:00Z',
    has_active_session: true,
  },
  {
    id: '01JTYRING2',
    name: '技术架构组',
    role: 'member',
    storage_mode: 'local',
    member_count: 3,
    node_count: 8,
    last_activity_at: '2026-04-16T14:00:00Z',
    has_active_session: false,
  },
  {
    id: '01JTYRING3',
    name: '项目管理组',
    role: 'admin',
    storage_mode: 'local',
    member_count: 7,
    node_count: 21,
    last_activity_at: '2026-04-15T10:00:00Z',
    has_active_session: false,
  },
]

export const MOCK_MEMBERS: Member[] = [
  { token_id: 'user-001', display_name: 'Kai', avatar: '🦊', role: 'creator', joined_at: '2026-04-15T00:00:00Z', online: true },
  { token_id: 'user-002', display_name: 'Alice', avatar: '🐱', role: 'admin', joined_at: '2026-04-15T01:00:00Z', online: true },
  { token_id: 'user-003', display_name: 'Bob', avatar: null, role: 'member', joined_at: '2026-04-16T00:00:00Z', online: false },
  { token_id: 'user-004', display_name: 'Carol', avatar: '🌟', role: 'member', joined_at: '2026-04-16T02:00:00Z', online: true },
  { token_id: 'user-005', display_name: 'Dave', avatar: null, role: 'readonly', joined_at: '2026-04-17T00:00:00Z', online: false },
]

export const MOCK_MESSAGES: ChatMessage[] = [
  {
    id: 'msg-001',
    role: 'user',
    sender_name: 'Kai',
    content: '帮我看看 #竞品分析 里最近的内容',
    node_refs: ['01JTYN1'],
    tag_refs: ['竞品分析'],
    created_at: '2026-04-17T08:30:00Z',
  },
  {
    id: 'msg-002',
    role: 'group_ring',
    sender_name: 'GROUP RING',
    content: '根据 #竞品分析 节点的内容，最近有以下更新：\n\n1. **竞品 A** 发布了 v3.0 版本\n2. **竞品 B** 调整了定价策略\n3. **竞品 C** 新增了 AI 功能模块\n\n建议重点关注竞品 C 的 AI 功能，可能影响我们的产品路线图。',
    created_at: '2026-04-17T08:30:05Z',
  },
  {
    id: 'msg-003',
    role: 'user',
    sender_name: 'Kai',
    content: '归档这段到 #竞品动态 下面',
    node_refs: ['01JTYN2'],
    tag_refs: ['竞品动态'],
    created_at: '2026-04-17T08:31:00Z',
  },
  {
    id: 'msg-004',
    role: 'system',
    sender_name: 'SYSTEM',
    content: '已归档到「竞品动态」节点。commit: a1b2c3d',
    created_at: '2026-04-17T08:31:03Z',
  },
]

export const MOCK_SESSION: Session = {
  id: '01JTYSESS',
  ring_id: '01JTYRING1',
  title: '竞品 A 深度讨论',
  description: '讨论竞品 A 的最新功能更新',
  skill: 'decision',
  phase: 'discussion',
  owner: 'user-001',
  archivable: true,
  archive_enabled: true,
  summary: null,
  created_at: '2026-04-17T08:00:00Z',
  updated_at: '2026-04-17T08:00:00Z',
}
