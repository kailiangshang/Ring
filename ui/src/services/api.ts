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

async function getToken(): Promise<string | null> {
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
  options: RequestInit = {},
): Promise<T> {
  const token = await getToken()
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  }
  if (token) {
    headers['X-Ring-Token'] = token
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
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
  get: <T>(path: string) => request<T>(path),

  post: <T>(path: string, body: unknown) =>
    request<T>(path, { method: 'POST', body: JSON.stringify(body) }),

  put: <T>(path: string, body: unknown) =>
    request<T>(path, { method: 'PUT', body: JSON.stringify(body) }),

  delete: <T>(path: string) =>
    request<T>(path, { method: 'DELETE' }),
}

export async function triggerArchiveSSE(
  ringId: string,
  input: import('../types/archive').CreateArchiveInput,
  onProgress: (event: import('../types/archive').ArchiveProgressEvent) => void,
  onComplete: () => void,
  onError: (err: string) => void,
): Promise<void> {
  const token = await getToken()
  const resp = await fetch(`${API_BASE}/rings/${ringId}/archive`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Ring-Token': token || '',
    },
    body: JSON.stringify(input),
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

async function exportFile(path: string, defaultFilename: string) {
  const token = await getToken()
  const res = await fetch(`${API_BASE}${path}`, {
    headers: {
      'X-Ring-Token': token || '',
    },
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

export async function exportRingChat(ringId: string) {
  return exportFile(`/rings/${ringId}/export/chat`, `ring_${ringId}_chat.md`)
}

export async function exportSelfChat() {
  return exportFile('/self/export/chat', 'self_chat.md')
}

export async function exportSuperChat() {
  return exportFile('/super/export/chat', 'super_chat.md')
}

export async function exportRingGraph(ringId: string) {
  return exportFile(`/rings/${ringId}/export/graph`, `ring_${ringId}_graph.json`)
}

export async function exportRingBackup(ringId: string) {
  return exportFile(`/rings/${ringId}/export/backup`, `ring_${ringId}_backup.json`)
}

export async function exportSessionMessages(ringId: string, sessionId: string) {
  return exportFile(`/rings/${ringId}/sessions/${sessionId}/export`, `session_${sessionId}.md`)
}
