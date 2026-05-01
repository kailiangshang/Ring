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
  const initialScrollDone = useRef(false)

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

    // Initial scroll to bottom on first render
    if (!initialScrollDone.current) {
      ref.current.scrollTop = ref.current.scrollHeight
      initialScrollDone.current = true
      return
    }

    // Auto-scroll during streaming if user was at bottom
    if (autoScroll && wasAtBottom.current) {
      ref.current.scrollTop = ref.current.scrollHeight
    }
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
