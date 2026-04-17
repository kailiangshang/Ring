import { useCallback, useRef } from 'react'

interface Position {
  x: number
  y: number
}

export function useDrag(
  onMove: (pos: Position) => void,
  bounds?: { width: number; height: number },
) {
  const startRef = useRef<{ mx: number; my: number; px: number; py: number } | null>(null)

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()
      const target = e.currentTarget as HTMLElement
      const rect = target.getBoundingClientRect()
      startRef.current = {
        mx: e.clientX,
        my: e.clientY,
        px: rect.left,
        py: rect.top,
      }

      const handleMove = (ev: MouseEvent) => {
        if (!startRef.current) return
        const dx = ev.clientX - startRef.current.mx
        const dy = ev.clientY - startRef.current.my
        let x = startRef.current.px + dx
        let y = startRef.current.py + dy
        if (bounds) {
          x = Math.max(0, Math.min(x, window.innerWidth - bounds.width))
          y = Math.max(0, Math.min(y, window.innerHeight - bounds.height))
        }
        onMove({ x, y })
      }

      const handleUp = () => {
        startRef.current = null
        document.removeEventListener('mousemove', handleMove)
        document.removeEventListener('mouseup', handleUp)
      }

      document.addEventListener('mousemove', handleMove)
      document.addEventListener('mouseup', handleUp)
    },
    [onMove, bounds],
  )

  return { onMouseDown }
}
