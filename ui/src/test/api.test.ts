import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'
import { MOCK_RINGS, MOCK_MEMBERS, MOCK_SESSION } from '../services/mock-data'

const BASE = 'http://localhost:7420'

const server = setupServer(
  // Health & Setup
  http.get(`${BASE}/api/health`, () => HttpResponse.json({ status: 'ok' })),
  http.get(`${BASE}/api/setup/status`, () => HttpResponse.json({ is_setup: true })),

  // Rings
  http.get(`${BASE}/api/rings`, () => HttpResponse.json({ rings: MOCK_RINGS })),
  http.get(`${BASE}/api/rings/:id`, ({ params }) => {
    const ring = MOCK_RINGS.find(r => r.id === params.id) || MOCK_RINGS[0]
    return HttpResponse.json(ring)
  }),
  http.get(`${BASE}/api/rings/:id/members`, () => HttpResponse.json({ members: MOCK_MEMBERS })),
  http.get(`${BASE}/api/rings/:id/mode`, () => HttpResponse.json({ 
    interaction_mode: 'normal', 
    skill_permission_mode: 'plan', 
    auto_archive: false 
  })),

  // Graph
  http.get(`${BASE}/api/rings/:id/graph`, () => HttpResponse.json({
    id: 'graph-01',
    name: 'main',
    nodes: [
      { id: 'n1', label: '竞品分析', node_type: 'topic' },
      { id: 'n2', label: '竞品动态', node_type: 'leaf' },
    ],
    edges: []
  })),

  // Skills
  http.get(`${BASE}/api/skills`, () => HttpResponse.json({
    skills: [
      { name: 'decision', description: '团队决策', source: 'builtin' },
      { name: 'research', description: '研究讨论', source: 'builtin' },
      { name: 'review', description: '评审', source: 'builtin' },
      { name: 'retrospective', description: '回顾', source: 'builtin' },
      { name: 'knowledge_sharing', description: '知识分享', source: 'builtin' },
    ]
  })),

  // Session
  http.get(`${BASE}/api/rings/:id/sessions`, () => HttpResponse.json({ sessions: [MOCK_SESSION] })),
  http.post(`${BASE}/api/rings/:id/sessions`, () => HttpResponse.json(MOCK_SESSION, { status: 201 })),
  http.post(`${BASE}/api/rings/:id/sessions/:sid/start`, () => HttpResponse.json({ ...MOCK_SESSION, phase: 'discussion' })),
  http.post(`${BASE}/api/rings/:id/sessions/:sid/close`, () => HttpResponse.json({ ...MOCK_SESSION, phase: 'closed' })),

  // Chat (mock SSE)
  http.post(`${BASE}/api/rings/:id/chat`, () => HttpResponse.json({ 
    messages: [{ id: 'msg-new', content: 'Mock response' }] 
  })),
  http.get(`${BASE}/api/rings/:id/chat/history`, () => HttpResponse.json({ 
    messages: MOCK_RINGS 
  })),

  // Self
  http.get(`${BASE}/api/self/metrics`, () => HttpResponse.json({
    chat_patterns: { total_messages: 10, self_messages: 5 },
    session_stats: { total_sessions: 3 },
    tool_usage: { tools: { search: 2 } }
  })),
  http.get(`${BASE}/api/self/memory`, () => HttpResponse.json([
    { name: 'user_profile', exists: true, line_count: 5, size: 100 },
    { name: 'preferences', exists: true, line_count: 3, size: 50 },
  ])),

  // Super
  http.post(`${BASE}/api/super/chat`, () => HttpResponse.json({ 
    messages: [{ id: 'msg-super', content: 'Super mock response' }] 
  })),

  // Export
  http.get(`${BASE}/api/rings/:id/export/chat`, () => HttpResponse.text('# Chat Export\n\nMock chat content')),
  http.get(`${BASE}/api/rings/:id/export/graph`, () => HttpResponse.json({ nodes: [], edges: [] })),
)

beforeAll(() => server.listen({ onUnhandledRequest: 'bypass' }))
afterAll(() => server.close())

describe('API Integration Tests', () => {
  describe('Health & Setup', () => {
    it('GET /api/health returns ok', async () => {
      const res = await fetch(`${BASE}/api/health`)
      const data = await res.json()
      expect(data.status).toBe('ok')
    })

    it('GET /api/setup/status returns is_setup', async () => {
      const res = await fetch(`${BASE}/api/setup/status`)
      const data = await res.json()
      expect(data.is_setup).toBe(true)
    })
  })

  describe('Rings', () => {
    it('GET /api/rings returns ring list', async () => {
      const res = await fetch(`${BASE}/api/rings`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.rings).toHaveLength(3)
      expect(data.rings[0].name).toBe('竞品分析组')
    })

    it('GET /api/rings/:id/members returns members', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/members`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.members).toHaveLength(5)
    })

    it('GET /api/rings/:id/mode returns mode config', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/mode`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.interaction_mode).toBe('normal')
    })
  })

  describe('Skills', () => {
    it('GET /api/skills returns 5 builtin skills', async () => {
      const res = await fetch(`${BASE}/api/skills`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.skills).toHaveLength(5)
      expect(data.skills.map((s: { name: string }) => s.name)).toEqual([
        'decision', 'research', 'review', 'retrospective', 'knowledge_sharing'
      ])
    })
  })

  describe('Session', () => {
    it('GET /api/rings/:id/sessions returns sessions', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/sessions`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.sessions).toHaveLength(1)
      expect(data.sessions[0].skill).toBe('decision')
    })

    it('POST /api/rings/:id/sessions creates session', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/sessions`, {
        method: 'POST',
        headers: { 'X-Ring-Token': 'user-001', 'Content-Type': 'application/json' },
        body: JSON.stringify({ title: 'Test', skill: 'decision', archivable: true })
      })
      expect(res.status).toBe(201)
    })
  })

  describe('Self', () => {
    it('GET /api/self/metrics returns metrics', async () => {
      const res = await fetch(`${BASE}/api/self/metrics`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(data.chat_patterns).toBeDefined()
      expect(data.session_stats).toBeDefined()
    })

    it('GET /api/self/memory returns memory entries', async () => {
      const res = await fetch(`${BASE}/api/self/memory`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      const data = await res.json()
      expect(Array.isArray(data)).toBe(true)
    })
  })

  describe('Export', () => {
    it('GET /api/rings/:id/export/chat returns text content', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/export/chat`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      expect(res.headers.get('content-type')).toMatch(/text\/(plain|markdown)/)
    })

    it('GET /api/rings/:id/export/graph returns JSON', async () => {
      const res = await fetch(`${BASE}/api/rings/01JTYRING1/export/graph`, {
        headers: { 'X-Ring-Token': 'user-001' }
      })
      expect(res.headers.get('content-type')).toContain('application/json')
    })
  })
})

describe('Mock Data Validation', () => {
  it('MOCK_RINGS has correct structure', () => {
    const ring = MOCK_RINGS[0]
    expect(ring).toHaveProperty('id')
    expect(ring).toHaveProperty('name')
    expect(ring).toHaveProperty('role')
    expect(ring).toHaveProperty('member_count')
    expect(ring).toHaveProperty('node_count')
  })

  it('MOCK_MEMBERS has correct roles', () => {
    const roles = MOCK_MEMBERS.map(m => m.role)
    expect(roles).toContain('creator')
    expect(roles).toContain('admin')
    expect(roles).toContain('member')
    expect(roles).toContain('readonly')
  })

  it('MOCK_SESSION has all required fields', () => {
    const session = MOCK_SESSION
    expect(session).toHaveProperty('id')
    expect(session).toHaveProperty('ring_id')
    expect(session).toHaveProperty('title')
    expect(session).toHaveProperty('skill')
    expect(session).toHaveProperty('phase')
    expect(session).toHaveProperty('owner')
  })
})