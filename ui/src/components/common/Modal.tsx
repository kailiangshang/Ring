import type { ReactNode } from 'react'

interface ModalProps {
  open: boolean
  on_close: () => void
  children: ReactNode
}

export function Modal({ open, on_close, children }: ModalProps) {
  if (!open) return null

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1000,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: 'rgba(0, 0, 0, 0.5)',
        }}
        onClick={on_close}
      />
      <div
        style={{
          position: 'relative',
          zIndex: 1,
          width: '100%',
          maxWidth: 480,
          maxHeight: '90vh',
          overflowY: 'auto',
          background: 'var(--bg-panel)',
          border: '1px solid var(--border)',
          borderRadius: 8,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  )
}
