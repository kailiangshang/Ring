import { create } from 'zustand'
import { api } from '../services/api'

export interface Notification {
  id: string
  user_id: string
  ring_id: string | null
  notification_type: string
  title: string
  content: string | null
  is_read: boolean
  related_id: string | null
  created_at: string
}

interface NotificationState {
  notifications: Notification[]
  unreadCount: number
  loading: boolean
  fetchNotifications: (unreadOnly?: boolean) => Promise<void>
  fetchUnreadCount: () => Promise<void>
  markAsRead: (id: string) => Promise<void>
  markAllAsRead: () => Promise<void>
  deleteNotification: (id: string) => Promise<void>
}

export const useNotificationStore = create<NotificationState>((set, get) => ({
  notifications: [],
  unreadCount: 0,
  loading: false,

  fetchNotifications: async (unreadOnly = false) => {
    set({ loading: true })
    try {
      const res = await api.get<Notification[]>(
        `/notifications?unread_only=${unreadOnly}`
      )
      set({ notifications: res })
    } catch {
      // keep existing
    }
    set({ loading: false })
  },

  fetchUnreadCount: async () => {
    try {
      const res = await api.get<{ count: number }>('/notifications/unread-count')
      set({ unreadCount: res.count })
    } catch {
      // keep existing
    }
  },

  markAsRead: async (id: string) => {
    await api.post(`/notifications/${id}/read`, {})
    set({
      notifications: get().notifications.map((n) =>
        n.id === id ? { ...n, is_read: true } : n
      ),
      unreadCount: Math.max(0, get().unreadCount - 1),
    })
  },

  markAllAsRead: async () => {
    await api.post('/notifications/read-all', {})
    set({
      notifications: get().notifications.map((n) => ({ ...n, is_read: true })),
      unreadCount: 0,
    })
  },

  deleteNotification: async (id: string) => {
    await api.delete(`/notifications/${id}`)
    const wasUnread = get().notifications.find((n) => n.id === id)?.is_read === false
    set({
      notifications: get().notifications.filter((n) => n.id !== id),
      unreadCount: wasUnread ? Math.max(0, get().unreadCount - 1) : get().unreadCount,
    })
  },
}))
