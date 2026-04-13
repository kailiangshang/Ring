import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useNotificationStore } from './notificationStore'

vi.mock('../api/client', () => ({
  list_notifications: vi.fn().mockResolvedValue([
    { id: 'n1', ring_id: 'ring-1', user_id: 'u1', type: 'invite', title: 'Li joined', body: null, related_id: null, is_read: false, created_at: '2026-04-01T00:00:00Z' },
    { id: 'n2', ring_id: 'ring-1', user_id: 'u1', type: 'pr', title: 'PR merged', body: null, related_id: null, is_read: true, created_at: '2026-04-01T00:00:00Z' },
  ]),
  mark_notification_read: vi.fn().mockResolvedValue(undefined),
}))

beforeEach(() => {
  useNotificationStore.setState({ notifications: [], loading: false, error: null })
})

describe('notificationStore', () => {
  it('loads notifications from API', async () => {
    await useNotificationStore.getState().load_notifications()
    const state = useNotificationStore.getState()
    expect(state.notifications).toHaveLength(2)
    expect(state.loading).toBe(false)
  })

  it('marks a notification as read', async () => {
    await useNotificationStore.getState().load_notifications()
    await useNotificationStore.getState().mark_read('n1')
    const n = useNotificationStore.getState().notifications.find((n) => n.id === 'n1')
    expect(n!.is_read).toBe(true)
  })

  it('marks all notifications as read', async () => {
    await useNotificationStore.getState().load_notifications()
    await useNotificationStore.getState().mark_all_read()
    const unread = useNotificationStore.getState().notifications.filter((n) => !n.is_read)
    expect(unread).toHaveLength(0)
  })
})
