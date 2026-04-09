import { useState } from 'react'
import { Input } from '../ui/Input'
import { Button } from '../ui/Button'
import './ChatInput.css'

interface ChatInputProps {
  on_send: (content: string) => void
  disabled?: boolean
}

export function ChatInput({ on_send, disabled }: ChatInputProps) {
  const [value, set_value] = useState('')

  const handle_submit = (e: React.FormEvent) => {
    e.preventDefault()
    const trimmed = value.trim()
    if (!trimmed || disabled) return
    on_send(trimmed)
    set_value('')
  }

  return (
    <form className="chat-input-form" onSubmit={handle_submit}>
      <Input
        type="text"
        value={value}
        onChange={(e) => set_value(e.target.value)}
        placeholder="Type a message..."
        disabled={disabled}
        className="chat-input-field"
      />
      <Button type="submit" disabled={disabled || !value.trim()}>Send</Button>
    </form>
  )
}
