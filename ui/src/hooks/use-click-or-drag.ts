import { useCallback, useRef } from 'react'

export function useClickOrDrag(
  onClick: () => void,
  onDragStart?: (e: React.MouseEvent) => void,
  threshold = 4,
) {
  const startRef = useRef<{ x: number; y: number; dragging: boolean } | null>(null)

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      startRef.current = { x: e.clientX, y: e.clientY, dragging: false }

      const handleMove = (ev: MouseEvent) => {
        if (!startRef.current) return
        const dx = ev.clientX - startRef.current.x
        const dy = ev.clientY - startRef.current.y
        if (Math.sqrt(dx * dx + dy * dy) > threshold) {
          if (!startRef.current.dragging) {
            startRef.current.dragging = true
            onDragStart?.(e)
          }
        }
      }

      const handleUp = () => {
        if (startRef.current && !startRef.current.dragging) {
          onClick()
        }
        startRef.current = null
        document.removeEventListener('mousemove', handleMove)
        document.removeEventListener('mouseup', handleUp)
      }

      document.addEventListener('mousemove', handleMove)
      document.addEventListener('mouseup', handleUp)
    },
    [onClick, onDragStart, threshold],
  )

  return { onMouseDown }
}
