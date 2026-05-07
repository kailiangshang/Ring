import { useState, useEffect, useRef } from 'react'
import { useNotificationStore, type Notification } from '../stores/notification-store'
import { useRingStore } from '../stores/ring-store'
import { useAppStore } from '../stores/app-store'
import { ConfirmModal } from './common/ConfirmModal'

export function NotificationBell() {
  const [isOpen, setIsOpen] = useState(false)
  const [confirmDialog, setConfirmDialog] = useState<{ message: string; action: () => void } | null>(null)
  const wrapperRef = useRef<HTMLDivElement>(null)
  const { notifications, unreadCount, fetchNotifications, fetchUnreadCount, markAsRead, markAllAsRead, deleteNotification } = useNotificationStore()
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const selectRing = useRingStore((s) => s.selectRing)
  const setContext = useAppStore((s) => s.setContext)

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

  useEffect(() => {
    if (!isOpen) return
    const handleClickOutside = (e: MouseEvent) => {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setIsOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [isOpen])

  useEffect(() => {
    if (!isOpen) return
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setIsOpen(false)
    }
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [isOpen])

  const handleNotificationClick = (n: Notification) => {
    if (!n.is_read) {
      markAsRead(n.id)
    }
    if (n.ring_id && n.ring_id !== active_ring_id) {
      selectRing(n.ring_id)
      setContext('ring')
    }
    setIsOpen(false)
  }

  return (
    <div ref={wrapperRef} style={{ position: 'relative' }}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        aria-label={`Notifications${unreadCount > 0 ? ` (${unreadCount} unread)` : ''}`}
        style={{
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          padding: '4px 8px',
          fontSize: 16,
          position: 'relative',
        }}
      >
        {'🔔'}
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
              {'Notifications'}
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
                {'Mark all read'}
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
              {'No notifications'}
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
                onClick={() => handleNotificationClick(notification)}
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
                      setConfirmDialog({
                        message: 'Delete this notification?',
                        action: () => deleteNotification(notification.id),
                      })
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
                    {'×'}
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      )}
      <ConfirmModal
        open={confirmDialog !== null}
        title="Confirm"
        message={confirmDialog?.message ?? ''}
        on_confirm={() => { confirmDialog?.action(); setConfirmDialog(null) }}
        on_cancel={() => setConfirmDialog(null)}
      />
    </div>
  )
}
