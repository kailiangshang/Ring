import { useState } from 'react'
import { useChatStore } from '../../stores/chat-store'
import { useCommandHistoryStore } from '../../stores/command-history-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'
import { CommandAutocomplete, useAutocompleteStore } from './CommandAutocomplete'

export function InputArea() {
  const { input, setInput, send, sending, stopStreaming, messages } = useChatStore()
  const ac = useAutocompleteStore()
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [showArchiveBanner, setShowArchiveBanner] = useState(false)

  const lastMessage = messages[messages.length - 1]
  const shouldRecommend = lastMessage && lastMessage.role === 'group_ring' && lastMessage.content && (
    lastMessage.content.includes('结论') ||
    lastMessage.content.includes('总结') ||
    lastMessage.content.includes('决策') ||
    lastMessage.content.includes('方案') ||
    lastMessage.content.includes('决定') ||
    lastMessage.content.includes('agreed') ||
    lastMessage.content.includes('decided') ||
    lastMessage.content.includes('conclusion') ||
    lastMessage.content.includes('resolved') ||
    lastMessage.content.includes('solution') ||
    lastMessage.content.includes('finalized') ||
    lastMessage.content.includes('确定') ||
    lastMessage.content.includes('共识')
  )

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

  const handleArchiveConfirm = () => {
    setShowArchiveBanner(false)
  }

  const handleArchiveDismiss = () => {
    setShowArchiveBanner(false)
  }

  return (
    <div style={{ position: 'relative' }}>
      {shouldRecommend && !showArchiveBanner && (
        <div
          style={{
            padding: '8px 12px',
            background: 'var(--bg-elevated)',
            borderTop: '1px solid var(--border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: 8,
          }}
        >
          <span style={{ fontSize: 12, color: 'var(--text-secondary)' }}>
            AI recommends archiving this conversation
          </span>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              onClick={handleArchiveConfirm}
              style={{
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 4,
                padding: '4px 12px',
                fontSize: 11,
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Archive
            </button>
            <button
              onClick={handleArchiveDismiss}
              style={{
                background: 'transparent',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '4px 12px',
                fontSize: 11,
                cursor: 'pointer',
              }}
            >
              Dismiss
            </button>
          </div>
        </div>
      )}
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
