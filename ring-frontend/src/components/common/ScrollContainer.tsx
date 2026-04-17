import type { ReactNode, CSSProperties } from 'react'

interface ScrollContainerProps {
  children: ReactNode
  style?: CSSProperties
  className?: string
}

export function ScrollContainer({ children, style, className }: ScrollContainerProps) {
  return (
    <div
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
