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
