import { useEffect, useCallback } from 'react'
import { Outlet, useNavigate } from 'react-router-dom'
import { HubNavBar } from './HubNavBar'
import type { NotificationItem } from '../ui/NotificationBell'
import { notification_to_item } from '../ui/NotificationBell'
import { useNotificationStore } from '../../stores/notificationStore'
import './AppShell.css'

export function AppShell() {
  const notifications = useNotificationStore((s) => s.notifications)
  const load_notifications = useNotificationStore((s) => s.load_notifications)
  const mark_read = useNotificationStore((s) => s.mark_read)
  const navigate = useNavigate()

  useEffect(() => { load_notifications() }, [load_notifications])

  const items: NotificationItem[] = notifications.map(notification_to_item)

  const on_notification_click = useCallback((item: NotificationItem) => {
    mark_read(item.id)
    if (item.target_path) navigate(item.target_path)
  }, [mark_read, navigate])

  return (
    <div className="app-shell">
      <HubNavBar notifications={items} on_notification_click={on_notification_click} />
      <Outlet />
    </div>
  )
}
