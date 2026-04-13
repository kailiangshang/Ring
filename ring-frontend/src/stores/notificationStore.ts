import { create } from 'zustand'
import * as api from '../api/client'
import type { Notification } from '../types'

interface NotificationState {
  notifications: Notification[]
  loading: boolean
  error: string | null

  load_notifications: () => Promise<void>
  mark_read: (notification_id: string) => Promise<void>
  mark_all_read: () => Promise<void>
}

export const useNotificationStore = create<NotificationState>((set, get) => ({
  notifications: [],
  loading: false,
  error: null,

  load_notifications: async () => {
    set({ loading: true, error: null })
    try {
      const notifications = await api.list_notifications()
      set({ notifications, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  mark_read: async (notification_id) => {
    try {
      await api.mark_notification_read(notification_id)
      set((s) => ({
        notifications: s.notifications.map((n) =>
          n.id === notification_id ? { ...n, is_read: true } : n,
        ),
      }))
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  mark_all_read: async () => {
    const unread = get().notifications.filter((n) => !n.is_read)
    await Promise.all(unread.map((n) => api.mark_notification_read(n.id)))
    set((s) => ({
      notifications: s.notifications.map((n) => ({ ...n, is_read: true })),
    }))
  },
}))
