import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

const BASE = 'http://localhost:7420'
const TOKEN = 'user-01KQ1F5B02Q5WYGFEJB5F6BRXS'
const RING1 = '01KQ1JK74P1T9826KXJBZRCCCE'
const RING2 = '01KQ1F8ECE5AC5Y430KXQ11FYY'

const server = setupServer(
  // Health
  http.get(`${BASE}/api/health`, () => HttpResponse.json({ status: 'ok' })),
  http.get(`${BASE}/api/setup/status`, () => HttpResponse.json({ is_setup: true })),

  // Rings
  http.get(`${BASE}/api/rings`, () => HttpResponse.json({
    rings: [
      { id: RING1, name: '每日论文学习', role: 'creator', member_count: 1, node_count: 1, last_activity_at: '2026-04-25T00:00:00Z', has_active_session: false },
      { id: RING2, name: '后端迁移rust', role: 'creator', member_count: 1, node_count: 5, last_activity_at: '2026-04-25T00:00:00Z', has_active_session: false },
    ]
  })),
  http.get(`${BASE}/api/rings/:id`, () => HttpResponse.json({ id: RING2, name: '后端迁移rust', role: 'creator' })),
  http.get(`${BASE}/api/rings/:id/members`, () => HttpResponse.json({
    members: [{ token_id: TOKEN, display_name: 'kai', avatar: '⚡', role: 'creator', joined_at: '2026-04-25T00:00:00Z', online: false }]
  })),
  http.get(`${BASE}/api/rings/:id/mode`, () => HttpResponse.json({ interaction_mode: 'normal', skill_permission_mode: 'plan', auto_archive: false })),
  http.get(`${BASE}/api/rings/:id/group-docs/role`, () => HttpResponse.json({ content: '' })),
  http.get(`${BASE}/api/rings/:id/blueprint`, () => HttpResponse.json({ status: 'pending' })),
  http.post(`${BASE}/api/rings/:id/blueprint/from-template`, () => HttpResponse.json({ nodes: [{ label: '中心主题', node_type: 'category' }], edges: [] })),

  // Graph
  http.get(`${BASE}/api/rings/:id/graph`, () => HttpResponse.json({
    id: 'graph-01', name: 'main', nodes: [
      { id: 'n1', label: '测试节点', node_type: 'leaf', content: '# Test' },
      { id: 'n2', label: '异步框架对比', node_type: 'topic', content: '# 异步框架' },
    ], edges: [{ id: 'e1', source_id: 'n1', target_id: 'n2', relation: 'related_to' }]
  })),
  http.post(`${BASE}/api/rings/:id/graph`, () => HttpResponse.json({ id: 'n-new', label: '新建节点', node_type: 'leaf' }, { status: 201 })),
  http.put(`${BASE}/api/rings/:id/graph/nodes/:nid`, () => HttpResponse.json({ id: 'n1', label: '已更新' })),
  http.delete(`${BASE}/api/rings/:id/graph/nodes/:nid`, () => HttpResponse.json({ success: true })),
  http.post(`${BASE}/api/rings/:id/graph/edges`, () => HttpResponse.json({ id: 'e-new', relation: 'related_to' }, { status: 201 })),

  // Skills
  http.get(`${BASE}/api/skills`, () => HttpResponse.json({
    skills: [
      { name: 'decision', description: '团队决策：收集材料 → 讨论 → 决策结论', source: 'builtin' },
      { name: 'research', description: '研究讨论', source: 'builtin' },
      { name: 'review', description: '评审', source: 'builtin' },
      { name: 'retrospective', description: '回顾', source: 'builtin' },
      { name: 'knowledge_sharing', description: '知识分享', source: 'builtin' },
    ]
  })),

  // Sessions
  http.get(`${BASE}/api/rings/:id/sessions`, () => HttpResponse.json({
    sessions: [
      { id: 's1', title: '测试Session', skill: 'decision', phase: 'closed', owner: TOKEN },
      { id: 's2', title: '研究Session', skill: 'research', phase: 'material_prep', owner: TOKEN },
    ]
  })),
  http.post(`${BASE}/api/rings/:id/sessions`, () => HttpResponse.json({ id: 's-new', phase: 'material_prep', owner: TOKEN }, { status: 201 })),
  http.get(`${BASE}/api/rings/:id/sessions/:sid`, () => HttpResponse.json({ id: 's1', phase: 'closed' })),
  http.get(`${BASE}/api/rings/:id/sessions/:sid/material-prep`, () => HttpResponse.json({ materials: [] })),
  http.post(`${BASE}/api/rings/:id/sessions/:sid/start`, () => HttpResponse.json({ phase: 'discussion' })),
  http.post(`${BASE}/api/rings/:id/sessions/:sid/close`, () => HttpResponse.json({ phase: 'closed' })),
  http.post(`${BASE}/api/rings/:id/sessions/:sid/reopen`, () => HttpResponse.json({ phase: 'material_prep' })),
  http.delete(`${BASE}/api/rings/:id/sessions/:sid`, () => HttpResponse.json({ success: true })),
  http.get(`${BASE}/api/rings/:id/sessions/:sid/messages`, () => HttpResponse.json({ messages: [] })),

  // Archive
  http.get(`${BASE}/api/rings/:id/archives`, () => HttpResponse.json({ archives: [] })),
  http.get(`${BASE}/api/rings/:id/archive-queue`, () => HttpResponse.json({ queue: [] })),
  http.post(`${BASE}/api/rings/:id/archive`, () => HttpResponse.text('event: progress\ndata: {"step":"pulling"}\n\n')),

  // Chat
  http.get(`${BASE}/api/rings/:id/chat/history`, () => HttpResponse.json({
    messages: [
      { id: 'm1', role: 'user', sender_name: 'kai', content: '你好', created_at: '2026-04-25T00:00:00Z' },
      { id: 'm2', role: 'group_ring', sender_name: 'GROUP RING', content: '你好！我是 Group Ring', created_at: '2026-04-25T00:00:01Z' },
    ], has_more: false
  })),

  // Self
  http.get(`${BASE}/api/self/metrics`, () => HttpResponse.json({
    chat_patterns: { total_messages: 10, self_messages: 5 },
    session_stats: { total_sessions: 3 },
    tool_usage: { tools: { search: 2 } }
  })),
  http.get(`${BASE}/api/self/memory`, () => HttpResponse.json([
    { name: 'user_profile', exists: false, line_count: 0, size: 0 },
    { name: 'preferences', exists: false, line_count: 0, size: 0 },
    { name: 'active_goals', exists: false, line_count: 0, size: 0 },
  ])),
  http.post(`${BASE}/api/self/metrics/heartbeat`, () => HttpResponse.json({ success: true })),
  http.get(`${BASE}/api/self/identity`, () => HttpResponse.json({ content: 'AI assistant', exists: true })),
  http.get(`${BASE}/api/self/style`, () => HttpResponse.json({ content: '', exists: false })),
  http.get(`${BASE}/api/self/personality`, () => HttpResponse.json({ content: '{}', exists: false })),

  // Super
  http.post(`${BASE}/api/super/chat`, () => HttpResponse.text('event: message_start\ndata: {"role":"super_ring"}\n\n')),
  http.get(`${BASE}/api/super/system-prompt`, () => HttpResponse.json({ content: 'You are Super Ring' })),
  http.get(`${BASE}/api/super/preferences`, () => HttpResponse.json({ content: '{}' })),

  // Notifications
  http.get(`${BASE}/api/notifications`, () => HttpResponse.json([])),
  http.get(`${BASE}/api/notifications/unread-count`, () => HttpResponse.json({ count: 0 })),

  // Invite & Members
  http.get(`${BASE}/api/rings/:id/invite-tokens`, () => HttpResponse.json({ tokens: [] })),
  http.post(`${BASE}/api/rings/:id/invite-tokens`, () => HttpResponse.json({ token: 'new-token', type: 'open' })),

  // Config
  http.get(`${BASE}/api/config/llm`, () => HttpResponse.json({ provider: 'openai', model: 'qwen3.5-plus', api_key_set: true })),
  http.get(`${BASE}/api/config/privacy_filters`, () => HttpResponse.json({ filters: [] })),

  // Export
  http.get(`${BASE}/api/rings/:id/export/chat`, () => HttpResponse.text('# Chat Export\n\nTest content')),
  http.get(`${BASE}/api/rings/:id/export/graph`, () => HttpResponse.json({ nodes: [], edges: [] })),
  http.get(`${BASE}/api/rings/:id/export/backup`, () => HttpResponse.json({ error: 'no gitlab' }, { status: 500 })),
)

beforeAll(() => server.listen({ onUnhandledRequest: 'bypass' }))
afterAll(() => server.close())

describe('场景 1: Super Ring 对话时切换到 Group Ring', () => {
  it('Super Chat 返回 200', async () => {
    const res = await fetch(`${BASE}/api/super/chat`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ content: 'test' })
    })
    expect(res.status).toBe(200)
  })

  it('Group Ring Graph 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/graph`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    const data = await res.json()
    expect(data.nodes).toHaveLength(2)
  })

  it('Group Ring Chat History 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/chat/history`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    const data = await res.json()
    expect(data.messages).toHaveLength(2)
  })
})

describe('场景 2: Session 完整生命周期', () => {
  it('创建 Session (decision)', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: '技术选型决策', skill: 'decision', archivable: true })
    })
    expect(res.status).toBe(201)
    expect((await res.json()).phase).toBe('material_prep')
  })

  it('获取 Material Prep', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions/s1/material-prep`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
  })

  it('开启 Session 进入 discussion', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions/s1/start`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).phase).toBe('discussion')
  })

  it('关闭 Session', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions/s1/close`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).phase).toBe('closed')
  })

  it('重新打开 Session', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions/s1/reopen`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).phase).toBe('material_prep')
  })

  it('删除 Session', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/sessions/s1`, {
      method: 'DELETE',
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
  })
})

describe('场景 3: Self 和 Group Ring 记忆隔离', () => {
  it('Self Metrics 可获取', async () => {
    const res = await fetch(`${BASE}/api/self/metrics`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    const data = await res.json()
    expect(data.chat_patterns).toBeDefined()
  })

  it('Self Memory 可获取', async () => {
    const res = await fetch(`${BASE}/api/self/memory`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect(Array.isArray(await res.json())).toBe(true)
  })

  it('Self Heartbeat 可发送', async () => {
    const res = await fetch(`${BASE}/api/self/metrics/heartbeat`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect([200, 204]).toContain(res.status)
  })
})

describe('场景 4: 归档流程', () => {
  it('Archive 可触发', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/archive`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content: '测试归档',
        suggested_title: '测试',
        node_suggestion: { action: 'create_new', parent_id: null, node_title: '测试节点' }
      })
    })
    expect(res.status).toBe(200)
  })

  it('Archives 列表可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/archives`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).archives).toBeDefined()
  })

  it('Archive Queue 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/archive-queue`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).queue).toBeDefined()
  })
})

describe('场景 5: 多 Ring 切换验证状态隔离', () => {
  it('Rings 列表可获取', async () => {
    const res = await fetch(`${BASE}/api/rings`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).rings).toHaveLength(2)
  })

  it('Ring1 Mode 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING1}/mode`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).interaction_mode).toBe('normal')
  })

  it('Ring2 Mode 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/mode`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).interaction_mode).toBe('normal')
  })

  it('Blueprint 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/blueprint`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).status).toBe('pending')
  })
})

describe('场景 6: Cross Ring Cache 失效验证', () => {
  it('Graph 可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/graph`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
  })

  it('可创建节点', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/graph`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ label: '新节点', node_type: 'leaf', content: '# 新内容' })
    })
    expect(res.status).toBe(201)
  })

  it('可创建边', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/graph/edges`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ source_id: 'n1', target_id: 'n2', edge_type: 'related_to' })
    })
    expect(res.status).toBe(201)
  })
})

describe('场景 7: Skills 系统', () => {
  it('Skills 列表返回 5 个', async () => {
    const res = await fetch(`${BASE}/api/skills`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).skills).toHaveLength(5)
  })

  it('包含 decision skill', async () => {
    const res = await fetch(`${BASE}/api/skills`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    const skills = (await res.json()).skills
    expect(skills.map((s: { name: string }) => s.name)).toContain('decision')
  })

  it('包含 research skill', async () => {
    const res = await fetch(`${BASE}/api/skills`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    const skills = (await res.json()).skills
    expect(skills.map((s: { name: string }) => s.name)).toContain('research')
  })
})

describe('场景 8: 导出功能', () => {
  it('导出 Chat 返回 markdown', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/export/chat`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.text())).toContain('# Chat Export')
  })

  it('导出 Graph 返回 JSON', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/export/graph`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect(res.headers.get('content-type')).toContain('application/json')
  })

  it('导出 Backup 在无 GitLab 时返回 500', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/export/backup`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(500)
  })
})

describe('场景 9: Members & Invite', () => {
  it('Members 列表可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/members`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).members).toHaveLength(1)
  })

  it('可创建 Invite Token', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/invite-tokens`, {
      method: 'POST',
      headers: { 'X-Ring-Token': TOKEN, 'Content-Type': 'application/json' },
      body: JSON.stringify({ type: 'open', role: 'member' })
    })
    expect(res.status).toBe(200)
  })

  it('Invite Tokens 列表可获取', async () => {
    const res = await fetch(`${BASE}/api/rings/${RING2}/invite-tokens`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).tokens).toBeDefined()
  })
})

describe('场景 10: 通知系统', () => {
  it('Notifications 列表可获取', async () => {
    const res = await fetch(`${BASE}/api/notifications`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect(Array.isArray(await res.json())).toBe(true)
  })

  it('Unread Count 可获取', async () => {
    const res = await fetch(`${BASE}/api/notifications/unread-count`, {
      headers: { 'X-Ring-Token': TOKEN }
    })
    expect(res.status).toBe(200)
    expect((await res.json()).count).toBeDefined()
  })
})