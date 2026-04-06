import { useState } from 'react'

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
    <form onSubmit={handle_submit} style={{ display: 'flex', gap: 8 }}>
      <input
        type="text"
        value={value}
        onChange={(e) => set_value(e.target.value)}
        placeholder="Type a message..."
        disabled={disabled}
        style={{ flex: 1 }}
      />
      <button type="submit" disabled={disabled || !value.trim()}>
        Send
      </button>
    </form>
  )
}
