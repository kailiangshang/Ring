import { useState, useEffect } from 'react'
import { useNotificationStore, type Notification } from '../stores/notification-store'

export function NotificationBell() {
  const [isOpen, setIsOpen] = useState(false)
  const { notifications, unreadCount, fetchNotifications, fetchUnreadCount, markAsRead, markAllAsRead, deleteNotification } = useNotificationStore()

  useEffect(() => {
    fetchUnreadCount()
    const interval = setInterval(fetchUnreadCount, 30000)
    return () => clearInterval(interval)
  }, [fetchUnreadCount])

  useEffect(() => {
    if (isOpen) {
      fetchNotifications()
    }
  }, [isOpen, fetchNotifications])

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        style={{
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          padding: '4px 8px',
          fontSize: 16,
          position: 'relative',
        }}
      >
        {"\ud83d\udd14"}
        {unreadCount > 0 && (
          <span
            style={{
              position: 'absolute',
              top: 0,
              right: 0,
              background: 'var(--accent-amber)',
              color: 'var(--bg-base)',
              fontSize: 10,
              fontWeight: 700,
              padding: '1px 4px',
              borderRadius: 8,
              minWidth: 16,
              textAlign: 'center',
            }}
          >
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div
          style={{
            position: 'absolute',
            top: '100%',
            right: 0,
            width: 320,
            maxHeight: 400,
            background: 'var(--bg-panel)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            zIndex: 1000,
            overflow: 'auto',
            boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
          }}
        >
          <div
            style={{
              padding: '8px 12px',
              borderBottom: '1px solid var(--border)',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--text-primary)' }}>
              {"Notifications"}
            </span>
            {unreadCount > 0 && (
              <button
                onClick={markAllAsRead}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--accent-cyan)',
                  fontSize: 11,
                  cursor: 'pointer',
                }}
              >
                {"Mark all read"}
              </button>
            )}
          </div>

          {notifications.length === 0 ? (
            <div
              style={{
                padding: 20,
                textAlign: 'center',
                color: 'var(--text-dim)',
                fontSize: 12,
              }}
            >
              {"No notifications"}
            </div>
          ) : (
            notifications.map((notification: Notification) => (
              <div
                key={notification.id}
                style={{
                  padding: '8px 12px',
                  borderBottom: '1px solid var(--border)',
                  background: notification.is_read ? 'transparent' : 'var(--bg-hover)',
                  cursor: 'pointer',
                }}
                onClick={() => {
                  if (!notification.is_read) {
                    markAsRead(notification.id)
                  }
                }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'flex-start',
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        fontSize: 11,
                        fontWeight: notification.is_read ? 400 : 700,
                        color: 'var(--text-primary)',
                      }}
                    >
                      {notification.title}
                    </div>
                    {notification.content && (
                      <div
                        style={{
                          fontSize: 10,
                          color: 'var(--text-secondary)',
                          marginTop: 2,
                        }}
                      >
                        {notification.content}
                      </div>
                    )}
                    <div
                      style={{
                        fontSize: 9,
                        color: 'var(--text-dim)',
                        marginTop: 4,
                      }}
                    >
                      {new Date(notification.created_at).toLocaleString()}
                    </div>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      deleteNotification(notification.id)
                    }}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--text-dim)',
                      fontSize: 14,
                      cursor: 'pointer',
                      padding: '0 4px',
                    }}
                  >
                    {"\u00d7"}
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}
