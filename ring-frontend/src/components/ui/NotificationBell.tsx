import { useState, useRef, useEffect } from 'react'
import './NotificationBell.css'

export interface NotificationItem {
  id: string
  title: string
  time: string
  target_path: string
}

interface NotificationBellProps {
  items: NotificationItem[]
  on_click: (item: NotificationItem) => void
}

export function NotificationBell({ items, on_click }: NotificationBellProps) {
  const [open, set_open] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) set_open(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open])

  const unread = items.length

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button className="notification-bell" onClick={() => set_open(!open)}>
        🔔
        {unread > 0 && <span className="notification-badge">{unread > 9 ? '9+' : unread}</span>}
      </button>
      {open && (
        <div className="notification-panel">
          <div className="notification-panel-header">通知</div>
          {items.length === 0 ? (
            <div className="notification-empty">没有新通知</div>
          ) : (
            items.map((item) => (
              <div key={item.id} className="notification-item" onClick={() => { on_click(item); set_open(false) }}>
                <div>
                  <div className="notification-item-title">{item.title}</div>
                  <div className="notification-item-time">{item.time}</div>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
