import { useSelfStore } from '../../stores/self-store'

export function SelfTrigger() {
  const { open, toggle, trigger_position, setTriggerPosition } = useSelfStore()

  const handleMouseDown = (e: React.MouseEvent) => {
    const startX = e.clientX
    const startY = e.clientY
    let moved = false

    const handleMove = (ev: MouseEvent) => {
      if (Math.abs(ev.clientX - startX) > 4 || Math.abs(ev.clientY - startY) > 4) {
        moved = true
        setTriggerPosition({
          x: Math.max(0, Math.min(ev.clientX - 14, window.innerWidth - 28)),
          y: Math.max(0, Math.min(ev.clientY - 14, window.innerHeight - 28)),
        })
      }
    }

    const handleUp = () => {
      if (!moved) toggle()
      document.removeEventListener('mousemove', handleMove)
      document.removeEventListener('mouseup', handleUp)
    }

    e.preventDefault()
    document.addEventListener('mousemove', handleMove)
    document.addEventListener('mouseup', handleUp)
  }

  return (
    <div
      onMouseDown={handleMouseDown}
      style={{
        position: 'fixed',
        left: trigger_position.x,
        top: trigger_position.y,
        width: 28,
        height: 28,
        borderRadius: '50%',
        background: open ? 'var(--accent-amber)' : 'var(--bg-panel)',
        border: '2px solid var(--accent-amber)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 14,
        cursor: 'pointer',
        zIndex: 1000,
        userSelect: 'none',
        touchAction: 'none',
      }}
    >
      🐱
    </div>
  )
}
