import type {
  SetupStatus,
  User,
  LlmConfig,
  GitlabConfig,
  Ring,
  RingListItem,
  CreateRingRequest,
} from '../types'

const BASE_URL = '/api/v1'

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error(body.error || `request failed: ${res.status}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export async function get_setup_status(): Promise<SetupStatus> {
  return request<SetupStatus>('/setup/status')
}

export async function set_username(display_name: string): Promise<User> {
  return request<User>('/setup/username', {
    method: 'POST',
    body: JSON.stringify({ display_name }),
  })
}

export async function set_llm(config: LlmConfig): Promise<void> {
  return request<void>('/setup/llm', {
    method: 'POST',
    body: JSON.stringify(config),
  })
}

export async function set_gitlab(config: GitlabConfig): Promise<void> {
  return request<void>('/setup/gitlab', {
    method: 'POST',
    body: JSON.stringify(config),
  })
}

export async function complete_setup(): Promise<void> {
  return request<void>('/setup/complete', { method: 'POST' })
}

export async function list_rings(): Promise<RingListItem[]> {
  const data = await request<{ rings: RingListItem[] }>('/rings')
  return data.rings
}

export async function create_ring(req: CreateRingRequest): Promise<Ring> {
  return request<Ring>('/rings', {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function get_ring(id: string): Promise<Ring> {
  return request<Ring>(`/rings/${id}`)
}

export async function delete_ring(id: string): Promise<void> {
  return request<void>(`/rings/${id}`, { method: 'DELETE' })
}
