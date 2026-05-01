import { useEffect, useRef, type ReactNode, type CSSProperties } from 'react'

interface ScrollContainerProps {
  children: ReactNode
  style?: CSSProperties
  className?: string
  autoScroll?: boolean
}

export function ScrollContainer({ children, style, className, autoScroll }: ScrollContainerProps) {
  const ref = useRef<HTMLDivElement>(null)
  const wasAtBottom = useRef(true)
  const lastScrollHeight = useRef(0)

  useEffect(() => {
    const el = ref.current
    if (!el) return

    const handleScroll = () => {
      wasAtBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < 80
    }

    el.addEventListener('scroll', handleScroll, { passive: true })
    return () => el.removeEventListener('scroll', handleScroll)
  }, [])

  useEffect(() => {
    if (!ref.current) return

    const currentHeight = ref.current.scrollHeight

    // If content height grew significantly (new messages loaded), scroll to bottom
    if (currentHeight > lastScrollHeight.current + 50) {
      ref.current.scrollTop = currentHeight
      lastScrollHeight.current = currentHeight
      return
    }

    // Auto-scroll during streaming if user was at bottom
    if (autoScroll && wasAtBottom.current) {
      ref.current.scrollTop = currentHeight
    }

    lastScrollHeight.current = currentHeight
  })

  return (
    <div
      ref={ref}
      className={className}
      style={{
        overflowY: 'auto',
        flex: 1,
        ...style,
      }}
    >
      {children}
    </div>
  )
}
