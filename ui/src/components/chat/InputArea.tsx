import { useState } from 'react'
import { useChatStore } from '../../stores/chat-store'
import { useCommandHistoryStore } from '../../stores/command-history-store'
import { useModeStore } from '../../stores/mode-store'
import { useAppStore } from '../../stores/app-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'
import { CommandAutocomplete, useAutocompleteStore } from './CommandAutocomplete'

export function InputArea() {
  const { input, setInput, send, sending, stopStreaming } = useChatStore()
  const ac = useAutocompleteStore()
  const [historyIndex, setHistoryIndex] = useState(-1)
  const auto_archive = useModeStore((s) => s.auto_archive)
  const toggleAutoArchive = useModeStore((s) => s.toggleAutoArchive)
  const context = useAppStore((s) => s.current_context)

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (ac.visible) {
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        ac.moveDown()
        return
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault()
        ac.moveUp()
        return
      }
      if (e.key === 'Enter') {
        const selected = ac.getSelected()
        if (selected) {
          e.preventDefault()
          setInput(selected)
          ac.hide()
          return
        }
      }
      if (e.key === 'Escape') {
        e.preventDefault()
        ac.hide()
        return
      }
    }

    // Command history navigation
    if (e.key === 'ArrowUp' && !e.shiftKey) {
      const history = useCommandHistoryStore.getState().getHistory()
      if (historyIndex < history.length - 1) {
        const newIndex = historyIndex + 1
        setHistoryIndex(newIndex)
        setInput(history[newIndex])
      }
      return
    }

    if (e.key === 'ArrowDown' && !e.shiftKey) {
      if (historyIndex > 0) {
        const newIndex = historyIndex - 1
        setHistoryIndex(newIndex)
        setInput(useCommandHistoryStore.getState().getHistory()[newIndex])
      } else if (historyIndex === 0) {
        setHistoryIndex(-1)
        setInput('')
      }
      return
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      if (input.trim()) {
        useCommandHistoryStore.getState().add(input.trim())
      }
      setHistoryIndex(-1)
      send()
    }
  }

  const handleChange = (val: string) => {
    setInput(val)
    ac.update(val)
    if (historyIndex !== -1) {
      setHistoryIndex(-1)
    }
  }

  const handleSelect = (val: string) => {
    setInput(val)
    ac.hide()
  }

  return (
    <div style={{ position: 'relative' }}>
      <CommandAutocomplete onSelect={handleSelect} />
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '8px 12px',
          borderTop: '1px solid var(--border)',
        }}
      >
        <ModeIndicator />
        {context === 'ring' && (
          <button
            onClick={toggleAutoArchive}
            style={{
              background: auto_archive ? 'var(--accent-green)' : 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 10px',
              color: auto_archive ? 'var(--bg-base)' : 'var(--text-secondary)',
              fontSize: 11,
              cursor: 'pointer',
              fontWeight: 700,
              whiteSpace: 'nowrap',
            }}
          >
            {auto_archive ? 'AUTO ON' : 'AUTO OFF'}
          </button>
        )}
        <input
          type="text"
          value={input}
          onChange={(e) => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={sending}
          placeholder="Type / for commands, @ to address..."
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '8px 12px',
            color: 'var(--text-primary)',
            fontSize: 13,
            fontFamily: 'inherit',
            outline: 'none',
            opacity: sending ? 0.6 : 1,
          }}
        />
        {sending ? (
          <button
            onClick={stopStreaming}
            style={{
              background: 'var(--accent-amber)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '8px 16px',
              fontSize: 12,
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            STOP
          </button>
        ) : (
          <button
            onClick={send}
            style={{
              background: 'var(--accent-cyan)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '8px 16px',
              fontSize: 12,
              fontWeight: 700,
              cursor: 'pointer',
              letterSpacing: '0.05em',
            }}
          >
            SEND
          </button>
        )}
      </div>
      <CommandHints />
    </div>
  )
}
