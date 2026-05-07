import { useCallback, useRef } from 'react'

interface Size {
  w: number
  h: number
}

export function useResize(
  onResize: (size: Size) => void,
  minSize?: { w: number; h: number },
) {
  const startRef = useRef<{ mx: number; my: number; sw: number; sh: number } | null>(null)

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()
      const target = (e.currentTarget as HTMLElement).parentElement!
      const rect = target.getBoundingClientRect()
      startRef.current = {
        mx: e.clientX,
        my: e.clientY,
        sw: rect.width,
        sh: rect.height,
      }

      const handleMove = (ev: MouseEvent) => {
        if (!startRef.current) return
        const dx = ev.clientX - startRef.current.mx
        const dy = ev.clientY - startRef.current.my
        let w = startRef.current.sw + dx
        let h = startRef.current.sh + dy
        if (minSize) {
          w = Math.max(minSize.w, w)
          h = Math.max(minSize.h, h)
        }
        w = Math.min(w, window.innerWidth - 10)
        h = Math.min(h, window.innerHeight - 10)
        onResize({ w, h })
      }

      const handleUp = () => {
        startRef.current = null
        document.removeEventListener('mousemove', handleMove)
        document.removeEventListener('mouseup', handleUp)
      }

      document.addEventListener('mousemove', handleMove)
      document.addEventListener('mouseup', handleUp)
    },
    [onResize, minSize],
  )

  return { onMouseDown }
}
