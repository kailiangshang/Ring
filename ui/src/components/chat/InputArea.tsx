import { useState, useRef } from 'react'
import { useChatStore } from '../../stores/chat-store'
import { useCommandHistoryStore } from '../../stores/command-history-store'
import { useArchiveStore } from '../../stores/archive-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import { useSelfStore } from '../../stores/self-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'
import { CommandAutocomplete, useAutocompleteStore } from './CommandAutocomplete'

export function InputArea() {
  const { input, setInput, send, sending, stopStreaming, messages } = useChatStore()
  const ac = useAutocompleteStore()
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [showArchiveBanner, setShowArchiveBanner] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

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
      const target = e.currentTarget as HTMLInputElement
      if (target.selectionStart !== 0 || target.selectionEnd !== 0) return
      const history = useCommandHistoryStore.getState().getHistory()
      if (historyIndex < history.length - 1) {
        const newIndex = historyIndex + 1
        setHistoryIndex(newIndex)
        setInput(history[newIndex])
      }
      return
    }

    if (e.key === 'ArrowDown' && !e.shiftKey) {
      const target = e.currentTarget as HTMLInputElement
      const len = target.value.length
      if (target.selectionStart !== len || target.selectionEnd !== len) return
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
    setInput('')
    ac.hide()

    const cmd = val.trim()
    if (cmd === '@self ') {
      useSelfStore.getState().setOpen(true)
      useSelfStore.getState().setTab('chat')
      setTimeout(() => {
        const el = document.querySelector<HTMLInputElement>('.self-chat-input')
        el?.focus()
      }, 50)
    } else if (cmd === '@super ') {
      useAppStore.getState().setActiveRing(null)
      setTimeout(() => inputRef.current?.focus(), 50)
    } else {
      setInput(val)
      setTimeout(() => inputRef.current?.focus(), 0)
    }
  }

  const handleArchiveConfirm = () => {
    const ring_id = useRingStore.getState().active_ring_id
    if (ring_id && lastMessage) {
      useArchiveStore.getState().triggerArchive(ring_id, lastMessage.content, 'Archive')
    }
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
          ref={inputRef}
          type="text"
          value={input}
          onChange={(e) => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
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
            disabled={!input.trim()}
            style={{
              background: input.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: input.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
              border: 'none',
              borderRadius: 4,
              padding: '8px 16px',
              fontSize: 12,
              fontWeight: 700,
              cursor: input.trim() ? 'pointer' : 'default',
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
