import { useRef, useEffect } from 'react'
import { useRingSuperStore } from '../../stores/ringSuperStore'
import { RingLogo } from '../ui/RingLogo'
import { ChatBubble } from '../chat/ChatBubble'
import { ChatInput } from '../chat/ChatInput'
import './RingSuperDrawer.css'

interface RingSuperDrawerProps {
  open: boolean
  on_close: () => void
}

export function RingSuperDrawer({ open, on_close }: RingSuperDrawerProps) {
  const messages = useRingSuperStore((s) => s.messages)
  const is_streaming = useRingSuperStore((s) => s.is_streaming)
  const error = useRingSuperStore((s) => s.error)
  const send_message = useRingSuperStore((s) => s.send_message)
  const bottom_ref = useRef<HTMLDivElement>(null)
  const container_ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottom_ref.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, is_streaming])

  useEffect(() => {
    if (!open) return
    const handler = (e: MouseEvent) => {
      if (container_ref.current && !container_ref.current.contains(e.target as Node)) on_close()
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [open, on_close])

  if (!open) return null

  return (
    <div ref={container_ref} className="ring-super-drawer">
      <div className="ring-super-drawer-header">
        <div className="ring-super-drawer-brand">
          <RingLogo size={16} />
          <span>Ring Super</span>
        </div>
        <button className="ring-super-drawer-close" onClick={on_close}>✕</button>
      </div>
      <div className="ring-super-drawer-messages">
        {messages.length === 0 && (
          <div className="ring-super-drawer-empty">全局助手，随时为你服务</div>
        )}
        {messages.map((msg) => (
          <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
        ))}
        {is_streaming && <div className="ring-super-drawer-typing">思考中...</div>}
        <div ref={bottom_ref} />
      </div>
      {error && <p className="ring-super-drawer-error">{error}</p>}
      <div className="ring-super-drawer-input">
        <ChatInput on_send={send_message} disabled={is_streaming} />
      </div>
    </div>
  )
}
