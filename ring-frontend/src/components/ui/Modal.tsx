import { useEffect } from 'react'
import './Modal.css'

interface ModalProps {
  open: boolean
  on_close: () => void
  title: string
  wide?: boolean
  children: React.ReactNode
  footer?: React.ReactNode
}

export function Modal({ open, on_close, title, wide, children, footer }: ModalProps) {
  useEffect(() => {
    if (!open) return
    const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') on_close() }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [open, on_close])

  if (!open) return null

  return (
    <div className="modal-overlay" onClick={on_close}>
      <div className={`modal-content${wide ? ' modal-wide' : ''}`} onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{title}</h3>
          <button className="modal-close" onClick={on_close}>&times;</button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-footer">{footer}</div>}
      </div>
    </div>
  )
}
