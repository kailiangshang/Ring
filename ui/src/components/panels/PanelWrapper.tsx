import { useEffect, useRef, useState, useCallback, type ReactNode } from 'react'

interface PanelWrapperProps {
  title: string
  depth: number
  onClose: () => void
  children: ReactNode
}

export function PanelWrapper({ title, depth, onClose, children }: PanelWrapperProps) {
  const [width, setWidth] = useState(320)
  const dragging = useRef(false)
  const startX = useRef(0)
  const startWidth = useRef(0)

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    dragging.current = true
    startX.current = e.clientX
    startWidth.current = width
    document.body.style.cursor = 'col-resize'
    document.body.style.userSelect = 'none'
  }, [width])

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!dragging.current) return
      const delta = startX.current - e.clientX
      const next = Math.min(700, Math.max(280, startWidth.current + delta))
      setWidth(next)
    }

    const handleMouseUp = () => {
      if (!dragging.current) return
      dragging.current = false
      document.body.style.cursor = ''
      document.body.style.userSelect = ''
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }
  }, [])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [onClose])

  const bgColors = ['var(--bg-panel)', '#0b1018', '#0c1220']
  const bg = bgColors[Math.min(depth - 1, 2)]

  return (
    <div
      style={{
        width,
        minWidth: 280,
        height: '100%',
        background: bg,
        borderLeft: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
        position: 'relative',
        flexShrink: 0,
      }}
    >
      <div
        onMouseDown={handleMouseDown}
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          bottom: 0,
          width: 5,
          cursor: 'col-resize',
          zIndex: 10,
        }}
      />
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 12px',
          borderBottom: '1px solid var(--border)',
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: '0.05em' }}>
          {title}
        </span>
        <button
          onClick={onClose}
          aria-label="Close panel"
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            fontSize: 14,
            padding: '0 4px',
          }}
        >
          ×
        </button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>{children}</div>
    </div>
  )
}
