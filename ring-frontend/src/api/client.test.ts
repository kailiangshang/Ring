import { describe, it, expect, vi, beforeEach } from 'vitest'
import {
  get_setup_status,
  set_username,
  set_llm,
  set_gitlab,
  complete_setup,
  list_rings,
  create_ring,
  get_ring,
  delete_ring,
} from './client'

beforeEach(() => {
  vi.restoreAllMocks()
})

function mock_fetch(response: unknown, status = 200) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: status >= 200 && status < 300,
      status,
      json: () => Promise.resolve(response),
    }),
  )
}

describe('get_setup_status', () => {
  it('calls correct url and returns data', async () => {
    const data = { setup_completed: false, step: 'username' }
    mock_fetch(data)
    const result = await get_setup_status()
    expect(fetch).toHaveBeenCalledWith('/api/v1/setup/status', expect.objectContaining({
      headers: { 'Content-Type': 'application/json' },
    }))
    expect(result).toEqual(data)
  })
})

describe('set_username', () => {
  it('posts display_name and returns user', async () => {
    const user = { user_id: 'abc', display_name: 'Test' }
    mock_fetch(user)
    const result = await set_username('Test')
    expect(fetch).toHaveBeenCalledWith('/api/v1/setup/username', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ display_name: 'Test' }),
    }))
    expect(result).toEqual(user)
  })
})

describe('set_llm', () => {
  it('posts llm config', async () => {
    mock_fetch(undefined, 204)
    await set_llm({ provider: 'openai', model: 'gpt-4', api_key: 'sk-xxx' })
    expect(fetch).toHaveBeenCalledWith('/api/v1/setup/llm', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ provider: 'openai', model: 'gpt-4', api_key: 'sk-xxx' }),
    }))
  })
})

describe('set_gitlab', () => {
  it('posts gitlab config', async () => {
    mock_fetch(undefined, 204)
    await set_gitlab({ repo_url: 'git@gitlab.com:x.git', auth_type: 'ssh_key' })
    expect(fetch).toHaveBeenCalledWith('/api/v1/setup/gitlab', expect.objectContaining({
      method: 'POST',
    }))
  })
})

describe('complete_setup', () => {
  it('posts to complete endpoint', async () => {
    mock_fetch(undefined, 204)
    await complete_setup()
    expect(fetch).toHaveBeenCalledWith('/api/v1/setup/complete', expect.objectContaining({
      method: 'POST',
    }))
  })
})

describe('list_rings', () => {
  it('fetches and unwraps rings array', async () => {
    const rings = [{ id: '1', name: 'Ring A', member_count: 5, graph_node_count: 10, last_activity_at: '', role: 'creator' }]
    mock_fetch({ rings })
    const result = await list_rings()
    expect(fetch).toHaveBeenCalledWith('/api/v1/rings', expect.any(Object))
    expect(result).toEqual(rings)
  })
})

describe('create_ring', () => {
  it('posts ring data', async () => {
    const ring = { id: '2', name: 'New Ring', status: 'blueprint_pending' }
    mock_fetch(ring)
    const result = await create_ring({ name: 'New Ring', description: 'desc' })
    expect(fetch).toHaveBeenCalledWith('/api/v1/rings', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ name: 'New Ring', description: 'desc' }),
    }))
    expect(result).toEqual(ring)
  })
})

describe('get_ring', () => {
  it('fetches ring by id', async () => {
    const ring = { id: 'abc', name: 'Test', status: 'active' }
    mock_fetch(ring)
    const result = await get_ring('abc')
    expect(fetch).toHaveBeenCalledWith('/api/v1/rings/abc', expect.any(Object))
    expect(result).toEqual(ring)
  })
})

describe('delete_ring', () => {
  it('deletes ring by id', async () => {
    mock_fetch(undefined, 204)
    await delete_ring('abc')
    expect(fetch).toHaveBeenCalledWith('/api/v1/rings/abc', expect.objectContaining({
      method: 'DELETE',
    }))
  })
})

describe('error handling', () => {
  it('throws on non-ok response', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: false,
        status: 400,
        json: () => Promise.resolve({ error: 'bad request' }),
      }),
    )
    await expect(set_username('')).rejects.toThrow('bad request')
  })
})
