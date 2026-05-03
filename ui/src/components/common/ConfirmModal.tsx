import { useEffect, useRef } from 'react'
import { Modal } from './Modal'

interface ConfirmModalProps {
  open: boolean
  title: string
  message: string
  on_confirm: () => void
  on_cancel: () => void
  confirm_label?: string
  cancel_label?: string
  variant?: 'danger' | 'default'
}

export function ConfirmModal({
  open,
  title,
  message,
  on_confirm,
  on_cancel,
  confirm_label = 'Confirm',
  cancel_label = 'Cancel',
  variant = 'default',
}: ConfirmModalProps) {
  const confirmRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (open) {
      setTimeout(() => confirmRef.current?.focus(), 0)
    }
  }, [open])

  return (
    <Modal open={open} on_close={on_cancel}>
      <div style={{ padding: 20 }}>
        <h3
          style={{
            margin: '0 0 12px 0',
            fontSize: 15,
            fontWeight: 600,
            color: 'var(--text-primary)',
          }}
        >
          {title}
        </h3>
        <p
          style={{
            margin: '0 0 20px 0',
            fontSize: 13,
            color: 'var(--text-secondary)',
            lineHeight: 1.5,
          }}
        >
          {message}
        </p>
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button
            onClick={on_cancel}
            style={{
              background: 'transparent',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 16px',
              fontSize: 12,
              cursor: 'pointer',
            }}
          >
            {cancel_label}
          </button>
          <button
            ref={confirmRef}
            onClick={on_confirm}
            style={{
              background:
                variant === 'danger' ? 'var(--accent-amber)' : 'var(--accent-cyan)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '6px 16px',
              fontSize: 12,
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            {confirm_label}
          </button>
        </div>
      </div>
    </Modal>
  )
}
