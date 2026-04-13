import type { ReactNode } from 'react'
import './FeaturePanelStrip.css'

interface FeaturePanelStripProps {
  open: boolean
  title: string
  on_close: () => void
  children: ReactNode
}

export function FeaturePanelStrip({ open, title, on_close, children }: FeaturePanelStripProps) {
  if (!open) return null

  return (
    <div className="feature-strip">
      <div className="feature-strip-header">
        <span className="feature-strip-title">{title}</span>
        <button className="feature-strip-close" onClick={on_close}>✕</button>
      </div>
      <div className="feature-strip-body">
        {children}
      </div>
    </div>
  )
}
