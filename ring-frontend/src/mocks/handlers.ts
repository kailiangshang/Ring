import { http, HttpResponse } from 'msw'
import type {
  RingListItem,
  Ring,
  Conversation,
  Message,
  GraphDetail,
  GraphNode,
  GraphEdge,
  BlueprintTemplate,
  PrListItem,
  Member,
  SessionData,
  InviteToken,
  SetupStatus,
  User,
} from '../types'

const RING_ID = 'ring-1'
const CONV_ID = 'conv-1'
const GRAPH_ID = 'graph-1'
const USER_ID = 'user-1'

const mock_rings: RingListItem[] = [
  {
    id: RING_ID,
    name: '日常学习',
    member_count: 3,
    graph_node_count: 12,
    last_activity_at: new Date().toISOString(),
    role: 'creator',
  },
  {
    id: 'ring-2',
    name: '产品竞品分析',
    member_count: 5,
    graph_node_count: 24,
    last_activity_at: new Date(Date.now() - 3600000).toISOString(),
    role: 'member',
  },
]

const mock_graph: GraphDetail = {
  graph_id: GRAPH_ID,
  nodes: [
    { id: 'n1', label: '竞品分析', node_type: 'category', parent_id: null, description: '根节点', graph_id: GRAPH_ID, markdown_path: null, created_at: '', updated_at: '' },
    { id: 'n2', label: '会议记录', node_type: 'category', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '', updated_at: '' },
    { id: 'n3', label: '技术对比', node_type: 'document', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '', updated_at: '' },
    { id: 'n4', label: '产品决策', node_type: 'document', parent_id: 'n1', description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '', updated_at: '' },
    { id: 'n5', label: '市场分析', node_type: 'category', parent_id: null, description: null, graph_id: GRAPH_ID, markdown_path: null, created_at: '', updated_at: '' },
  ],
  edges: [
    { id: 'e1', source_id: 'n2', target_id: 'n3', relation: 'produces', label: null, graph_id: GRAPH_ID },
  ],
}

const mock_conversations: Conversation[] = [
  { id: CONV_ID, ring_id: RING_ID, title: 'General', context_mode: 'auto', created_at: '' },
]

const mock_messages: Message[] = [
  { id: 'm1', conversation_id: CONV_ID, role: 'user', content: '帮我把今天的会议纪要整理一下', sender_id: USER_ID, created_at: '' },
  { id: 'm2', conversation_id: CONV_ID, role: 'assistant', content: '已整理完成。建议归档到：**会议记录 → 2026-04-10 周会**\n\n## 会议纪要\n\n1. Q2 目标确认\n2. 竞品分析进展\n3. 下周计划', sender_id: '', created_at: '' },
]

const mock_prs: PrListItem[] = [
  { pr_id: 1, title: '添加会议纪要 2026-04-10', author: 'Kai', state: 'opened' },
  { pr_id: 2, title: '更新技术对比文档', author: 'Li', state: 'merged' },
]

const mock_members: Member[] = [
  { id: 'mb1', ring_id: RING_ID, user_id: USER_ID, token_id: 1, display_name: 'Kai', role: 'creator', joined_at: '' },
  { id: 'mb2', ring_id: RING_ID, user_id: 'user-2', token_id: 2, display_name: 'Li', role: 'admin', joined_at: '' },
  { id: 'mb3', ring_id: RING_ID, user_id: 'user-3', token_id: 3, display_name: 'Ming', role: 'member', joined_at: '' },
]

const mock_sessions: SessionData[] = [
  { id: 'sess-1', ring_id: RING_ID, title: 'Q2 规划讨论', scenario: 'discussion', status: 'active', archive_enabled: true, member_count: 3, created_at: '' },
  { id: 'sess-2', ring_id: RING_ID, title: 'Deep Research', scenario: 'deep_research', status: 'closed', archive_enabled: false, member_count: 2, created_at: '' },
]

const mock_templates: BlueprintTemplate[] = [
  { id: 'tpl-1', name: '产品分析', description: '适用于产品竞品分析、市场调研', graphs: [{ name: '竞品树', graph_type: 'tree', categories: ['竞品', '功能', '价格'] }] },
  { id: 'tpl-2', name: '学习笔记', description: '适用于个人或团队学习', graphs: [{ name: '知识树', graph_type: 'tree', categories: ['主题', '章节', '笔记'] }] },
  { id: 'tpl-3', name: '项目管理', description: '适用于项目跟踪和文档管理', graphs: [{ name: '项目树', graph_type: 'tree', categories: ['阶段', '任务', '文档'] }] },
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
    const ring: Ring = { id: `ring-${ring_counter++}`, name: body.name, description: body.description, status: 'active' }
    mock_rings.push({ ...ring, member_count: 1, graph_node_count: 0, last_activity_at: new Date().toISOString(), role: 'creator' })
    return HttpResponse.json(ring)
  }),

  http.get(`${BASE}/rings/:ringId`, ({ params }) => {
    const r = mock_rings.find(r => r.id === params.ringId)
    if (!r) return new HttpResponse(null, { status: 404 })
    const ring: Ring = { id: r.id, name: r.name, description: undefined, status: 'active' }
    return HttpResponse.json(ring)
  }),

  http.delete(`${BASE}/rings/:ringId`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/conversations`, () => {
    return HttpResponse.json({ conversations: mock_conversations })
  }),

  http.post(`${BASE}/rings/:ringId/conversations`, async ({ request }) => {
    const body = await request.json() as { title: string }
    const conv: Conversation = { id: `conv-${Date.now()}`, ring_id: RING_ID, title: body.title, context_mode: 'auto', created_at: new Date().toISOString() }
    return HttpResponse.json(conv)
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
          `event: message\ndata: ${JSON.stringify({ type: 'done' })}\n\n`,
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
          `event: message\ndata: ${JSON.stringify({ type: 'blueprint_proposal', graphs: [{ name: '示例图谱', graph_type: 'tree', categories: ['类别1', '类别2'] }] })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'done' })}\n\n`,
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
    return HttpResponse.json({ graphs: [{ name: '示例图谱', graph_type: 'tree', categories: ['类别1'] }], preview: 'mock preview' })
  }),

  http.post(`${BASE}/rings/:ringId/blueprint/confirm`, () => {
    return HttpResponse.json({ success: true, message: 'Blueprint confirmed' })
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
    return HttpResponse.json(node)
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
    return HttpResponse.json({ node_id: params.nodeId, label: node?.label || 'Unknown', markdown_path: null, content: '# Mock Content\n\n这是节点的 Markdown 内容。', last_modified: new Date().toISOString() })
  }),

  http.post(`${BASE}/rings/:ringId/graphs/:graphId/edges`, async ({ request }) => {
    const body = await request.json() as { source_id: string; target_id: string; relation: string }
    const edge: GraphEdge = { id: `e-${Date.now()}`, source_id: body.source_id, target_id: body.target_id, relation: body.relation, label: null, graph_id: GRAPH_ID }
    return HttpResponse.json(edge)
  }),

  http.delete(`${BASE}/rings/:ringId/graphs/:graphId/edges/:edgeId`, () => new HttpResponse(null, { status: 204 })),

  http.post(`${BASE}/rings/:ringId/search`, () => {
    return HttpResponse.json({ results: [], total: 0 })
  }),

  http.post(`${BASE}/rings/:ringId/archive`, () => {
    return HttpResponse.json({ archive_id: 'arch-1', markdown_path: '.ring/docs/mock.md', git_status: 'pending', pr_url: null, queue_position: 1 })
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
    return HttpResponse.json({ pr_id: Number(params.prId), title: pr?.title || 'Mock PR', author: pr?.author || 'Unknown', changes: [{ file: '.ring/docs/mock.md', status: 'added', additions: 15, deletions: 0, diff: '+## Mock Diff\n+This is a mock diff content.\n+Added some lines.' }] })
  }),

  http.post(`${BASE}/rings/:ringId/git/prs/:prId/merge`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/git/prs/:prId/reject`, () => new HttpResponse(null, { status: 204 })),
  http.get(`${BASE}/rings/:ringId/git/commits`, () => HttpResponse.json({ commits: [] })),

  http.get(`${BASE}/rings/:ringId/members`, () => {
    return HttpResponse.json({ members: mock_members })
  }),

  http.post(`${BASE}/rings/:ringId/members/invites`, () => {
    const token: InviteToken = { token: 'mock-invite-token-abc123', token_type: 'open', role: 'member', max_uses: 10, used_count: 0, created_at: new Date().toISOString() }
    return HttpResponse.json(token)
  }),

  http.put(`${BASE}/rings/:ringId/members/:memberId/role`, () => new HttpResponse(null, { status: 204 })),
  http.delete(`${BASE}/rings/:ringId/members/:memberId`, () => new HttpResponse(null, { status: 204 })),

  http.get(`${BASE}/rings/:ringId/sessions`, () => {
    return HttpResponse.json({ sessions: mock_sessions })
  }),

  http.post(`${BASE}/rings/:ringId/sessions`, async ({ request }) => {
    const body = await request.json() as { title?: string; scenario: string }
    const session: SessionData = { id: `sess-${Date.now()}`, ring_id: RING_ID, title: body.title || body.scenario, scenario: body.scenario, status: 'active', archive_enabled: false, member_count: 1, created_at: new Date().toISOString() }
    return HttpResponse.json(session)
  }),

  http.get(`${BASE}/rings/:ringId/sessions/:sessionId`, ({ params }) => {
    const s = mock_sessions.find(s => s.id === params.sessionId) || mock_sessions[0]
    return HttpResponse.json(s)
  }),

  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/close`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/leave`, () => new HttpResponse(null, { status: 204 })),
  http.put(`${BASE}/rings/:ringId/sessions/:sessionId/archive-toggle`, () => new HttpResponse(null, { status: 204 })),
  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/invite`, () => new HttpResponse(null, { status: 204 })),
  http.delete(`${BASE}/rings/:ringId/sessions/:sessionId`, () => new HttpResponse(null, { status: 204 })),
  http.get(`${BASE}/rings/:ringId/sessions/:sessionId/messages`, () => HttpResponse.json({ messages: [] })),

  http.post(`${BASE}/rings/:ringId/sessions/:sessionId/messages`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'text', content: 'Mock session reply.' })}\n\n`))
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'done' })}\n\n`))
        controller.close()
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.post(`${BASE}/super-ring/chat`, () => {
    const stream = new ReadableStream({
      start(controller) {
        const encoder = new TextEncoder()
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'text', content: '我是 Super Ring 全局助手。有什么可以帮你的？' })}\n\n`))
        controller.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ type: 'done' })}\n\n`))
        controller.close()
      },
    })
    return new Response(stream, { headers: { 'Content-Type': 'text/event-stream' } })
  }),

  http.get(`${BASE}/settings`, () => HttpResponse.json(mock_settings)),
  http.put(`${BASE}/settings`, () => new HttpResponse(null, { status: 204 })),
]
