const API_BASE = '/api'

export class ApiError extends Error {
  status: number
  code: string
  constructor(status: number, code: string, message: string) {
    super(message)
    this.status = status
    this.code = code
  }
}

export async function getToken(): Promise<string | null> {
  return localStorage.getItem('ring_token')
}

export async function setToken(token: string): Promise<void> {
  localStorage.setItem('ring_token', token)
}

export async function clearToken(): Promise<void> {
  localStorage.removeItem('ring_token')
}

async function request<T>(
  path: string,
  options: RequestInit & { signal?: AbortSignal } = {},
): Promise<T> {
  const token = await getToken()
  const { signal, ...rest } = options
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(rest.headers as Record<string, string>),
  }
  if (token) {
    headers['X-Ring-Token'] = token
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...rest,
    headers,
    signal,
  })

  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(
      res.status,
      body?.error?.code ?? 'unknown',
      body?.error?.message ?? res.statusText,
    )
  }

  if (res.status === 204) return undefined as T
  return res.json()
}

export const api = {
  get: <T>(path: string, signal?: AbortSignal) => request<T>(path, { signal }),

  post: <T>(path: string, body: unknown, signal?: AbortSignal) =>
    request<T>(path, { method: 'POST', body: JSON.stringify(body), signal }),

  put: <T>(path: string, body: unknown, signal?: AbortSignal) =>
    request<T>(path, { method: 'PUT', body: JSON.stringify(body), signal }),

  delete: <T>(path: string, signal?: AbortSignal) =>
    request<T>(path, { method: 'DELETE', signal }),
}

export async function triggerArchiveSSE(
  ringId: string,
  input: import('../types/archive').CreateArchiveInput,
  onProgress: (event: import('../types/archive').ArchiveProgressEvent) => void,
  onComplete: () => void,
  onError: (err: string) => void,
  signal?: AbortSignal,
): Promise<void> {
  const token = await getToken()
  const resp = await fetch(`${API_BASE}/rings/${ringId}/archive`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Ring-Token': token || '',
    },
    body: JSON.stringify(input),
    signal,
  })

  if (!resp.ok || !resp.body) {
    onError(`archive failed: ${resp.status}`)
    return
  }

  const reader = resp.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() || ''
    for (const line of lines) {
      if (line.startsWith('data:')) {
        const data = line.slice(5).trim()
        if (!data) continue
        try {
          const parsed = JSON.parse(data)
          if (parsed.step && parsed.message) {
            onProgress(parsed)
          }
        } catch {
          /* skip */
        }
      }
    }
  }
  onComplete()
}

export async function getPreferences(): Promise<{ content: string; is_custom: boolean }> {
  return api.get('/super/preferences')
}

export async function updatePreferences(content: string): Promise<{ content: string; is_custom: boolean }> {
  return api.put('/super/preferences', { content })
}

export async function listSkills(): Promise<{ skills: import('../types/skill').SkillInfo[] }> {
  return api.get('/skills')
}

export async function installSkill(name: string, sourceUrl: string): Promise<import('../types/skill').InstallResult> {
  return api.post('/skills/install', { name, source_url: sourceUrl })
}

export async function getSkillDetail(name: string): Promise<import('../types/skill').SkillDetail> {
  return api.get(`/skills/${encodeURIComponent(name)}`)
}

export async function removeSkill(name: string): Promise<{ ok: boolean; name: string }> {
  return api.delete(`/skills/${encodeURIComponent(name)}`)
}

export async function createInviteToken(ring_id: string, input: import('../types/invite').CreateInviteInput): Promise<import('../types/invite').InviteToken> {
  return api.post(`/rings/${ring_id}/invite-tokens`, input)
}

export async function listInviteTokens(ring_id: string): Promise<{ tokens: import('../types/invite').InviteToken[] }> {
  return api.get(`/rings/${ring_id}/invite-tokens`)
}

export async function revokeInviteToken(ring_id: string, token: string): Promise<void> {
  return api.delete(`/rings/${ring_id}/invite-tokens/${token}`)
}

export async function listJoinRequests(ring_id: string, status = 'pending'): Promise<{ requests: import('../types/invite').JoinRequest[] }> {
  return api.get(`/rings/${ring_id}/join-requests?status=${status}`)
}

export async function approveJoinRequest(ring_id: string, request_id: string): Promise<{ ok: boolean; token_id: string; ring_name: string; role: string }> {
  return api.post(`/rings/${ring_id}/join-requests/${request_id}/approve`, {})
}

export async function rejectJoinRequest(ring_id: string, request_id: string, note?: string): Promise<{ ok: boolean }> {
  return api.post(`/rings/${ring_id}/join-requests/${request_id}/reject`, note ? { note } : {})
}

export async function verifyJoinToken(token: string): Promise<import('../types/invite').JoinInfo> {
  return api.get(`/join/info?token=${encodeURIComponent(token)}`)
}

export async function joinRing(invite_token: string, display_name: string): Promise<{ token_id: string; ring_id: string; ring_name: string; role: string; gitlab_repo_url: string | null }> {
  return api.post('/join', { invite_token, display_name })
}

export async function localJoin(invite_token: string, creator_ip: string): Promise<{ ok: boolean; ring_id: string; ring_name: string; role: string }> {
  return api.post('/join/local', { invite_token, creator_ip })
}

export async function applyJoin(invite_token: string, display_name: string, message?: string): Promise<{ request_id: string; status: string; ring_name: string }> {
  return api.post('/join/apply', { invite_token, display_name, message })
}

export async function checkApplyStatus(request_id: string): Promise<{ request_id: string; status: string; ring_name: string | null; ring_id: string | null; role: string | null; review_note: string | null; token_id: string | null }> {
  return api.get(`/join/apply/status?id=${encodeURIComponent(request_id)}`)
}

export async function testLLMConfig(input: { provider: string; model: string; api_key?: string; base_url?: string }): Promise<{ ok: boolean; message: string }> {
  return api.post('/config/llm/test', input)
}

function downloadFromResponse(res: Response, defaultFilename: string) {
  const disposition = res.headers.get('content-disposition')
  let filename = defaultFilename
  if (disposition) {
    const match = disposition.match(/filename="([^"]+)"/)
    if (match) filename = match[1]
  }
  res.blob().then((blob) => {
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    window.URL.revokeObjectURL(url)
  })
}

async function exportFile(path: string, defaultFilename: string, signal?: AbortSignal) {
  const token = await getToken()
  const res = await fetch(`${API_BASE}${path}`, {
    headers: {
      'X-Ring-Token': token || '',
    },
    signal,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(
      res.status,
      body?.error?.code ?? 'unknown',
      body?.error?.message ?? res.statusText,
    )
  }
  downloadFromResponse(res, defaultFilename)
}

export async function exportRingChat(ringId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/export/chat`, `ring_${ringId}_chat.md`, signal)
}

export async function exportSelfChat(signal?: AbortSignal) {
  return exportFile('/self/export/chat', 'self_chat.md', signal)
}

export async function exportSuperChat(signal?: AbortSignal) {
  return exportFile('/super/export/chat', 'super_chat.md', signal)
}

export async function exportRingGraph(ringId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/export/graph`, `ring_${ringId}_graph.json`, signal)
}

export async function exportRingBackup(ringId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/export/backup`, `ring_${ringId}_backup.tar.gz`, signal)
}

export async function exportAIReport(ringId: string, nodeIds: string[], topic?: string, signal?: AbortSignal) {
  const params = new URLSearchParams()
  params.set('node_ids', nodeIds.join(','))
  if (topic) params.set('topic', topic)
  return exportFile(`/rings/${ringId}/export/report?${params.toString()}`, `ring_${ringId}_report.md`, signal)
}

export async function exportSessionMessages(ringId: string, sessionId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/sessions/${sessionId}/export`, `session_${sessionId}.md`, signal)
}

export async function exportNodeMarkdown(ringId: string, nodeId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/export/node?node_id=${encodeURIComponent(nodeId)}`, `node.md`, signal)
}

export async function exportChatPdf(ringId: string, signal?: AbortSignal) {
  return exportFile(`/rings/${ringId}/export/chat-pdf`, `ring_${ringId}_chat.pdf`, signal)
}

export async function getGitLog(ringId: string, signal?: AbortSignal): Promise<{ commits: Array<{ sha: string; subject: string; author: string; date: string }> }> {
  const token = await getToken()
  const res = await fetch(`${API_BASE}/rings/${ringId}/repo/git-log`, {
    headers: { 'X-Ring-Token': token || '' },
    signal,
  })
  if (!res.ok) throw new ApiError(res.status, 'git_log_failed', res.statusText)
  return res.json()
}

export async function postGitRevert(ringId: string, sha: string, signal?: AbortSignal): Promise<{ reverted: string; new_commit: string }> {
  const token = await getToken()
  const res = await fetch(`${API_BASE}/rings/${ringId}/repo/revert`, {
    method: 'POST',
    headers: { 'X-Ring-Token': token || '', 'Content-Type': 'application/json' },
    body: JSON.stringify({ sha }),
    signal,
  })
  if (!res.ok) throw new ApiError(res.status, 'revert_failed', res.statusText)
  return res.json()
}

export async function uploadFile(path: string, file: File, signal?: AbortSignal): Promise<unknown> {
  const token = await getToken()
  const formData = new FormData()
  formData.append('file', file)
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: token ? { 'X-Ring-Token': token } : {},
    body: formData,
    signal,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(
      res.status,
      body?.error?.code ?? 'unknown',
      body?.error?.message ?? res.statusText,
    )
  }
  return res.json()
}

export async function crossRingQuery(query: string): Promise<{ content: string }> {
  return api.post('/super/cross-ring-query', { query })
}

export async function crossRingAnalysis(
  ringNames: string[],
  analysisType: 'compare' | 'merge' | 'summary',
  question?: string
): Promise<{ content: string }> {
  return api.post('/super/cross-ring-analysis', { ring_names: ringNames, analysis_type: analysisType, question })
}

export async function getArchiveDiff(ringId: string, archiveId: string): Promise<{ diffs: Array<{ old_path: string; new_path: string; diff: string }> }> {
  return api.get(`/rings/${ringId}/archives/${archiveId}/diff`)
}

export async function getSelfPersonality(): Promise<{ tone: string; proactivity: boolean; suggestions: boolean }> {
  return api.get('/self/personality')
}

export async function updateSelfPersonality(data: { tone: string; proactivity: boolean; suggestions: boolean }): Promise<{ ok: boolean }> {
  return api.put('/self/personality', data)
}

export async function getSelfPrivacy(): Promise<{ level: string; share_metrics: boolean; allow_proactive: boolean }> {
  return api.get('/self/privacy')
}

export async function updateSelfPrivacy(data: { level: string; share_metrics: boolean; allow_proactive: boolean }): Promise<{ ok: boolean }> {
  return api.put('/self/privacy', data)
}

export async function exportSelfData(): Promise<Record<string, unknown>> {
  return api.get('/self/export')
}

export async function getAutoCompact(): Promise<{ auto_compact: boolean }> {
  return api.get('/config/auto_compact')
}

export async function updateAutoCompact(auto_compact: boolean): Promise<{ auto_compact: boolean }> {
  return api.put('/config/auto_compact', { auto_compact })
}

export async function getTokenCount(ring_id?: string): Promise<{ total_tokens: number; threshold: number; warning_threshold: number }> {
  const path = ring_id ? `/rings/${ring_id}/tokens` : '/self/tokens'
  return api.get(path)
}

export async function resetSelfData(): Promise<{ ok: boolean }> {
  return api.post('/self/reset', {})
}

export async function getSyncBundle(
  ringId: string,
  signal?: AbortSignal,
): Promise<SyncBundle> {
  return api.get(`/rings/${ringId}/sync/bundle`, signal)
}

export interface SyncBundle {
  version: string
  ring_id: string
  exported_at: string
  graphs: Array<{
    graph: {
      id: string
      ring_id: string
      name: string
      created_at: string
      updated_at: string
    }
    nodes: Array<{
      id: string
      graph_id: string
      ring_id: string
      label: string
      parent_id: string | null
      node_type: string
      content: string
      tags: string
      markdown_path: string | null
      metadata: string
      created_at: string
      updated_at: string
    }>
    edges: Array<{
      id: string
      graph_id: string
      ring_id: string
      source_id: string
      target_id: string
      relation: string
      label: string
      created_at: string
    }>
  }>
  archive_records: Array<{
    id: string
    ring_id: string
    session_id: string | null
    node_id: string | null
    file_name: string
    commit_sha: string | null
    status: string
    archived_by: string
    created_at: string
    updated_at: string
  }>
  group_docs: Array<{
    doc_name: string
    content: string
    updated_at: string
  }>
  archive_files: Array<{
    file_name: string
    content: string
  }>
}

export async function postSyncImport(
  ringId: string,
  creatorIp: string,
  signal?: AbortSignal,
): Promise<{
  imported: {
    graphs: number
    nodes: number
    edges: number
    archive_records: number
    group_docs: number
    archive_files: number
  }
  exported_at: string
}> {
  return api.post(
    `/rings/sync/import`,
    { ring_id: ringId, creator_ip: creatorIp },
    signal,
  )
}
