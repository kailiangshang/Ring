import { useEffect, useRef, useState } from 'react'
import { Modal } from './Modal'

interface PromptModalProps {
  open: boolean
  title: string
  placeholder?: string
  default_value?: string
  on_submit: (value: string) => void
  on_cancel: () => void
}

export function PromptModal({
  open,
  title,
  placeholder = '',
  default_value = '',
  on_submit,
  on_cancel,
}: PromptModalProps) {
  const [value, set_value] = useState(default_value)
  const input_ref = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      set_value(default_value)
      setTimeout(() => input_ref.current?.focus(), 0)
    }
  }, [open, default_value])

  const handle_key_down = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (value.trim()) {
        on_submit(value.trim())
      }
    }
  }

  return (
    <Modal open={open} on_close={on_cancel}>
      <div style={{ padding: 20 }}>
        <h3
          style={{
            margin: '0 0 12px 0',
            fontSize: 15,
            fontWeight: 600,
            color: 'var(--text-primary)',
          }}
        >
          {title}
        </h3>
        <input
          ref={input_ref}
          type="text"
          value={value}
          onChange={(e) => set_value(e.target.value)}
          onKeyDown={handle_key_down}
          placeholder={placeholder}
          style={{
            width: '100%',
            boxSizing: 'border-box',
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '8px 12px',
            color: 'var(--text-primary)',
            fontSize: 13,
            fontFamily: 'inherit',
            outline: 'none',
            marginBottom: 16,
          }}
        />
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8 }}>
          <button
            onClick={on_cancel}
            style={{
              background: 'transparent',
              color: 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 16px',
              fontSize: 12,
              cursor: 'pointer',
            }}
          >
            Cancel
          </button>
          <button
            onClick={() => {
              if (value.trim()) on_submit(value.trim())
            }}
            disabled={!value.trim()}
            style={{
              background: value.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: value.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
              border: 'none',
              borderRadius: 4,
              padding: '6px 16px',
              fontSize: 12,
              fontWeight: 600,
              cursor: value.trim() ? 'pointer' : 'default',
            }}
          >
            OK
          </button>
        </div>
      </div>
    </Modal>
  )
}
