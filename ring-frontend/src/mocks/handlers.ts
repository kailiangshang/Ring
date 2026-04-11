import { http, HttpResponse } from 'msw'
import type {
  Ring,
  Conversation,
  Message,
  GraphDetail,
  GraphNode,
  GraphEdge,
  BlueprintTemplate,
  PrListItem,
  Member,
  SessionListItem,
  SessionDetail,
  InviteToken,
  SetupStatus,
  User,
  Notification,
} from '../types'

const RING_ID = 'ring-1'
const CONV_ID = 'conv-1'
const GRAPH_ID = 'graph-1'
const USER_ID = 'user-1'

const mock_rings: Ring[] = [
  {
    id: RING_ID,
    name: '日常学习',
    description: '日常学习记录',
    creator_id: USER_ID,
    gitlab_repo: 'git@gitlab.corp:user/learning.git',
    local_path: '/home/.ring/repos/ring-1',
    next_token_id: 3,
    status: 'active',
    created_at: '2026-04-01T00:00:00Z',
    updated_at: new Date().toISOString(),
  },
  {
    id: 'ring-2',
    name: '产品竞品分析',
    description: '产品竞品分析 Ring',
    creator_id: 'user-2',
    gitlab_repo: 'git@gitlab.corp:user/analysis.git',
    local_path: '/home/.ring/repos/ring-2',
    next_token_id: 5,
    status: 'active',
    created_at: '2026-04-01T00:00:00Z',
    updated_at: new Date(Date.now() - 3600000).toISOString(),
  },
]

const mock_graph: GraphDetail = {
  graph_id: GRAPH_ID,
  nodes: [
    { id: 'n1', label: '竞品分析', node_type: 'category', parent_id: null, description: '根节点', graph_id: GRAPH_ID, markdown_path: null, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
    { id: 'n2', label: '会议记录', node_type: 'category', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
    { id: 'n3', label: '技术对比', node_type: 'document', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
    { id: 'n4', label: '产品决策', node_type: 'document', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
    { id: 'n5', label: '市场分析', node_type: 'category', parent_id: null, description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
  ],
  edges: [
    { id: 'e1', source_id: 'n2', target_id: 'n3', relation: 'produces', label: null, graph_id: GRAPH_ID },
  ],
}

const mock_conversations: Conversation[] = [
  { id: CONV_ID, ring_id: RING_ID, title: 'General', mode: 'ring_group', context_mode: 'storage', token_count: 0, token_limit: 8000, auto_compact: false, summary: null, compacted_at: null, created_by: USER_ID, created_at: '2026-04-01T00:00:00Z', updated_at: '2026-04-01T00:00:00Z' },
]

const mock_messages: Message[] = [
  { id: 'm1', conversation_id: CONV_ID, role: 'user', content: '帮我把今天的会议纪要整理一下', sender_id: USER_ID, tool_calls: null, archived: false, created_at: '2026-04-01T00:00:00Z' },
  { id: 'm2', conversation_id: CONV_ID, role: 'assistant', content: '已整理完成。建议归档到：**会议记录 → 2026-04-10 周会**\n\n## 会议纪要\n\n1. Q2 目标确认\n2. 竞品分析进展\n3. 下周计划', sender_id: null, tool_calls: null, archived: false, created_at: '2026-04-01T00:00:00Z' },
]

const mock_prs: PrListItem[] = [
  { pr_id: 1, title: '添加会议纪要 2026-04-10', author: 'Kai', state: 'opened', changes: [{ file: '.ring/docs/mock.md', status: 'added', additions: 15, deletions: 0, diff: '+## Mock Diff\n+This is a mock diff content.' }] },
  { pr_id: 2, title: '更新技术对比文档', author: 'Li', state: 'merged', changes: [{ file: '.ring/docs/tech.md', status: 'modified', additions: 5, deletions: 2, diff: '-Old line\n+New line' }] },
]

const mock_members: Member[] = [
  { id: 'mb1', ring_id: RING_ID, user_id: USER_ID, token_id: 1, display_name: 'Kai', role: 'creator', joined_at: '2026-04-01T00:00:00Z' },
  { id: 'mb2', ring_id: RING_ID, user_id: 'user-2', token_id: 2, display_name: 'Li', role: 'admin', joined_at: '2026-04-01T00:00:00Z' },
  { id: 'mb3', ring_id: RING_ID, user_id: 'user-3', token_id: 3, display_name: 'Ming', role: 'member', joined_at: '2026-04-01T00:00:00Z' },
]

const mock_session_list: SessionListItem[] = [
  { id: 'sess-1', title: 'Q2 规划讨论', created_by: USER_ID, member_count: 3, archive_enabled: true, status: 'active', created_at: '2026-04-01T00:00:00Z' },
  { id: 'sess-2', title: 'Deep Research', created_by: USER_ID, member_count: 2, archive_enabled: false, status: 'closed', created_at: '2026-04-01T00:00:00Z' },
]

const mock_session_detail: SessionDetail = {
  id: 'sess-1',
  ring_id: RING_ID,
  title: 'Q2 规划讨论',
  scenario: 'discussion',
  created_by: USER_ID,
  archive_enabled: true,
  status: 'active',
  members: [
    { user_id: USER_ID, role: 'creator', status: 'active' },
    { user_id: 'user-2', role: 'member', status: 'active' },
    { user_id: 'user-3', role: 'member', status: 'active' },
  ],
  created_at: '2026-04-01T00:00:00Z',
}

const mock_templates: BlueprintTemplate[] = [
  { id: 'tpl-1', name: '产品分析', description: '适用于产品竞品分析、市场调研', graphs: JSON.stringify([{ name: '竞品树', graph_type: 'tree', categories: ['竞品', '功能', '价格'] }]), is_system: true, created_by: null, created_at: '2026-04-01T00:00:00Z' },
  { id: 'tpl-2', name: '学习笔记', description: '适用于个人或团队学习', graphs: JSON.stringify([{ name: '知识树', graph_type: 'tree', categories: ['主题', '章节', '笔记'] }]), is_system: true, created_by: null, created_at: '2026-04-01T00:00:00Z' },
  { id: 'tpl-3', name: '项目管理', description: '适用于项目跟踪和文档管理', graphs: JSON.stringify([{ name: '项目树', graph_type: 'tree', categories: ['阶段', '任务', '文档'] }]), is_system: true, created_by: null, created_at: '2026-04-01T00:00:00Z' },
]

const mock_notifications: Notification[] = [
  { id: 'notif-1', ring_id: RING_ID, user_id: USER_ID, type: 'invite', title: 'Li 加入了你的 Ring', body: null, related_id: null, is_read: false, created_at: '2026-04-01T00:00:00Z' },
]

const mock_settings = {
  llm_provider: 'openai',
  llm_model: 'gpt-4',
  llm_api_key: 'sk-***',
  llm_base_url: '',
  privacy_enabled: 'false',
}

const BASE = '/api/v1'

let ring_counter = 3

export const handlers = [
  http.get(`${BASE}/setup/status`, () => {
    const body: SetupStatus = { setup_completed: true, step: 'done', user_id: USER_ID }
    return HttpResponse.json(body)
  }),

  http.post(`${BASE}/setup/username`, async ({ request }) => {
    const body = await request.json() as { display_name: string }
    const user: User = { user_id: USER_ID, display_name: body.display_name }
    return HttpResponse.json(user)
  }),

  http.post(`${BASE}/setup/llm`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/setup/gitlab`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/setup/complete`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings`, () => {
    return HttpResponse.json({ rings: mock_rings })
  }),

  http.post(`${BASE}/rings`, async ({ request }) => {
    const body = await request.json() as { name: string; description?: string }
    const ring: Ring = {
      id: `ring-${ring_counter++}`,
      name: body.name,
      description: body.description || null,
      creator_id: USER_ID,
      gitlab_repo: 'git@gitlab.corp:user/mock.git',
      local_path: '/home/.ring/repos/mock',
      next_token_id: 1,
      status: 'active',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
    mock_rings.push(ring)
    return HttpResponse.json(ring, { status: 201 })
  }),

  http.get(`${BASE}/rings/:ringId`, ({ params }) => {
    const r = mock_rings.find(r => r.id === params.ringId)
    if (!r) return new HttpResponse(null, { status: 404 })
    return HttpResponse.json(r)
  }),

  http.delete(`${BASE}/rings/:ringId`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/conversations`, () => {
    return HttpResponse.json({ conversations: mock_conversations })
  }),

  http.post(`${BASE}/rings/:ringId/conversations`, async ({ request }) => {
    const body = await request.json() as { title: string }
    const conv: Conversation = {
      id: `conv-${Date.now()}`,
      ring_id: RING_ID,
      title: body.title,
      mode: 'ring_group',
      context_mode: 'storage',
      token_count: 0,
      token_limit: 8000,
      auto_compact: false,
      summary: null,
      compacted_at: null,
      created_by: USER_ID,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }
    return HttpResponse.json(conv, { status: 201 })
  }),

  http.get(`${BASE}/rings/:ringId/conversations/:convId/messages`, () => {
    return HttpResponse.json({ messages: mock_messages })
  }),

  http.post(`${BASE}/rings/:ringId/conversations/:convId/messages`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        const chunks = [
          `event: message\ndata: ${JSON.stringify({ type: 'text', content: '这是 mock 回复。在实际环境中，AI 会根据上下文生成回复。' })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'done', message_id: null, token_usage: null })}\n\n`,
        ]
        let i = 0
        const interval = setInterval(() => {
          if (i < chunks.length) {
            controller.enqueue(encoder.encode(chunks[i++]))
          } else {
            clearInterval(interval)
            controller.close()
          }
        }, 300)
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.get(`${BASE}/rings/:ringId/blueprint/templates`, () => {
    return HttpResponse.json({ templates: mock_templates })
  }),

  http.post(`${BASE}/rings/:ringId/blueprint/chat`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        const chunks = [
          `event: message\ndata: ${JSON.stringify({ type: 'text', content: '我建议创建以下图谱结构...' })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'blueprint_proposal', data: { title: '示例图谱' } })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'done', message_id: null, token_usage: null })}\n\n`,
        ]
        let i = 0
        const interval = setInterval(() => {
          if (i < chunks.length) {
            controller.enqueue(encoder.encode(chunks[i++]))
          } else {
            clearInterval(interval)
            controller.close()
          }
        }, 300)
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.post(`${BASE}/rings/:ringId/blueprint/preview`, () => {
    return HttpResponse.json({
      graphs: [
        {
          name: '示例图谱',
          nodes: [
            { id: 'pn1', label: '根节点', node_type: 'topic' },
            { id: 'pn2', label: '子节点', node_type: 'category' },
          ],
          edges: [
            { source_id: 'pn1', target_id: 'pn2', relation: 'contains' },
          ],
        },
      ],
    })
  }),

  http.post(`${BASE}/rings/:ringId/blueprint/confirm`, () => {
    return HttpResponse.json({
      blueprint_id: 'bp-mock-1',
      graphs: [{ id: 'graph-mock-1', name: '示例图谱', graph_type: 'topic' }],
      status: 'confirmed',
    })
  }),

  http.get(`${BASE}/rings/:ringId/graphs`, () => {
    return HttpResponse.json([GRAPH_ID])
  }),

  http.get(`${BASE}/rings/:ringId/graphs/:graphId`, () => {
    return HttpResponse.json(mock_graph)
  }),

  http.post(`${BASE}/rings/:ringId/graphs/:graphId/nodes`, async ({ request }) => {
    const body = await request.json() as { label: string; node_type: string }
    const node: GraphNode = { id: `n-${Date.now()}`, label: body.label, node_type: body.node_type, parent_id: null, description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    return HttpResponse.json(node, { status: 201 })
  }),

  http.put(`${BASE}/rings/:ringId/graphs/:graphId/nodes/:nodeId`, async ({ request, params }) => {
    const body = await request.json() as Record<string, string>
    const existing = mock_graph.nodes.find(n => n.id === params.nodeId)
    const node: GraphNode = { ...(existing || mock_graph.nodes[0]), ...body }
    return HttpResponse.json(node)
  }),

  http.delete(`${BASE}/rings/:ringId/graphs/:graphId/nodes/:nodeId`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/graphs/:graphId/nodes/:nodeId/content`, ({ params }) => {
    const node = mock_graph.nodes.find(n => n.id === params.nodeId)
    return HttpResponse.json({ node_id: params.nodeId as string, label: node?.label || 'Unknown', markdown_path: null, content: '# Mock Content\n\n这是节点的 Markdown 内容。', last_modified: new Date().toISOString() })
  }),

  http.post(`${BASE}/rings/:ringId/graphs/:graphId/edges`, async ({ request }) => {
    const body = await request.json() as { source_id: string; target_id: string; relation: string }
    const edge: GraphEdge = { id: `e-${Date.now()}`, source_id: body.source_id, target_id: body.target_id, relation: body.relation, label: null, graph_id: GRAPH_ID }
    return HttpResponse.json(edge, { status: 201 })
  }),

  http.delete(`${BASE}/rings/:ringId/graphs/:graphId/edges/:edgeId`, () => new HttpResponse(null, { status: 204 })),

  http.post(`${BASE}/rings/:ringId/search`, () => {
    return HttpResponse.json({ results: [], total: 0 })
  }),

  http.post(`${BASE}/rings/:ringId/archive`, () => {
    return HttpResponse.json({ archive_id: 'arch-1', markdown_path: '.ring/docs/mock.md', git_status: 'pending', pr_url: null, queue_position: 1 }, { status: 201 })
  }),

  http.get(`${BASE}/rings/:ringId/archive/queue`, () => {
    return HttpResponse.json({ current_review: null, queue: [] })
  }),

  http.post(`${BASE}/rings/:ringId/archive/:archiveId/confirm`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/git/prs`, () => {
    return HttpResponse.json({ prs: mock_prs })
  }),

  http.get(`${BASE}/rings/:ringId/git/prs/:prId/diff`, ({ params }) => {
    const pr = mock_prs.find(p => p.pr_id === Number(params.prId))
    return HttpResponse.json(pr || mock_prs[0])
  }),

  http.post(`${BASE}/rings/:ringId/git/prs/:prId/merge`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/git/prs/:prId/reject`, () => new HttpResponse(null, { status: 204 })),
  http.get(`${BASE}/rings/:ringId/git/commits`, () => HttpResponse.json({ commits: [] })),

  http.get(`${BASE}/rings/:ringId/members`, () => {
    return HttpResponse.json({ members: mock_members })
  }),

  http.post(`${BASE}/rings/:ringId/members/invites`, () => {
    const token: InviteToken = {
      id: 'inv-1',
      ring_id: RING_ID,
      token: 'mock-invite-token-abc123',
      token_type: 'open',
      role: 'member',
      inviter_id: USER_ID,
      max_uses: 10,
      use_count: 0,
      max_members: null,
      expires_at: new Date(Date.now() + 86400000).toISOString(),
      used_at: null,
      revoked_at: null,
      created_at: new Date().toISOString(),
    }
    return HttpResponse.json(token)
  }),

  http.put(`${BASE}/rings/:ringId/members/:memberId/role`, () => new HttpResponse(null, { status: 204 })),
  http.delete(`${BASE}/rings/:ringId/members/:memberId`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/sessions`, () => {
    return HttpResponse.json({ sessions: mock_session_list })
  }),

  http.post(`${BASE}/rings/:ringId/sessions`, async ({ request }) => {
    const body = await request.json() as { title?: string; scenario: string }
    const session: SessionDetail = {
      id: `sess-${Date.now()}`,
      ring_id: RING_ID,
      title: body.title || body.scenario,
      scenario: body.scenario,
      created_by: USER_ID,
      archive_enabled: false,
      status: 'active',
      members: [{ user_id: USER_ID, role: 'creator', status: 'active' }],
      created_at: new Date().toISOString(),
    }
    return HttpResponse.json(session, { status: 201 })
  }),

  http.get(`${BASE}/rings/:ringId/sessions/:sessionId`, () => {
    return HttpResponse.json(mock_session_detail)
  }),

  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/close`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/leave`, () => new HttpResponse(null, { status: 204 })),
  http.put(`${BASE}/rings/:ringId/sessions/:sessionId/archive-toggle`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/invite`, () => HttpResponse.json({ invited: [] })),
  http.delete(`${BASE}/rings/:ringId/sessions/:sessionId`, () => new HttpResponse(null, { status: 204 })),
  http.get(`${BASE}/rings/:ringId/sessions/:sessionId/messages`, () => HttpResponse.json({ messages: [] })),

  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/messages`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'text', content: 'Mock session reply.' })}\n\n`))
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'done', message_id: null, token_usage: null })}\n\n`))
        controller.close()
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.post(`${BASE}/ring-super/chat`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'text', content: '我是 Ring Super 全局助手。有什么可以帮你的？' })}\n\n`))
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'done', message_id: null, token_usage: null })}\n\n`))
        controller.close()
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.get(`${BASE}/settings`, () => HttpResponse.json(mock_settings)),
  http.put(`${BASE}/settings`, () => HttpResponse.json({ ok: true })),

  http.get(`${BASE}/notifications`, () => {
    return HttpResponse.json({ notifications: mock_notifications })
  }),
  http.post(`${BASE}/notifications/:notificationId`, () => new HttpResponse(null, { status: 204 })),
]
