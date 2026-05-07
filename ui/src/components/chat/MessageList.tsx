import { useState } from 'react'
import { useChatStore } from '../../stores/chat-store'
import { useArchiveStore } from '../../stores/archive-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import { MessageItem } from './MessageItem'
import { ScrollContainer } from '../common/ScrollContainer'

export function MessageList() {
  const messages = useChatStore((s) => s.messages)
  const sending = useChatStore((s) => s.sending)
  const selection_mode = useChatStore((s) => s.selection_mode)
  const selected_messages = useChatStore((s) => s.selected_messages)
  const clearSelection = useChatStore((s) => s.clearSelection)
  const deleteSelected = useChatStore((s) => s.deleteSelected)
  const [archiveDialog, setArchiveDialog] = useState(false)
  const [archiveNodeType, setArchiveNodeType] = useState<string>('topic')
  const [archiveTitle, setArchiveTitle] = useState('')

  const handleArchiveClick = () => {
    const selected = messages.filter((m) => selected_messages.includes(m.id))
    const userMsg = selected.find((m) => m.role === 'user')
    const title = userMsg?.content.slice(0, 40) || 'untitled'
    setArchiveTitle(title)
    setArchiveNodeType('topic')
    setArchiveDialog(true)
  }

  const handleArchiveConfirm = async () => {
    const selected = messages.filter((m) => selected_messages.includes(m.id))
    const content = selected
      .map((m) => `[${m.sender_name}]: ${m.content}`)
      .join('\n\n')
    const ringId = useRingStore.getState().active_ring_id
    const sessionId = useAppStore.getState().active_session_id ?? undefined
    if (ringId) {
      await useArchiveStore.getState().triggerArchive(ringId, content, archiveTitle, sessionId, archiveNodeType)
    }
    setArchiveDialog(false)
    clearSelection()
  }

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, minHeight: 0, position: 'relative' }}>
      {selection_mode && (
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '6px 12px',
          background: 'var(--bg-active)',
          borderBottom: '1px solid var(--accent-cyan)',
          fontSize: 12,
          color: 'var(--accent-cyan)',
          fontWeight: 700,
          zIndex: 10,
        }}>
          <span>{selected_messages.length} selected</span>
          <div style={{ display: 'flex', gap: 8 }}>
            {useRingStore.getState().active_ring_id && (
              <button
                onClick={handleArchiveClick}
                style={{
                  background: 'var(--accent-cyan)',
                  color: 'var(--bg-base)',
                  border: 'none',
                  borderRadius: 4,
                  padding: '3px 10px',
                  fontSize: 11,
                  fontWeight: 700,
                  cursor: 'pointer',
                }}
              >
                ARCHIVE
              </button>
            )}
            <button
              onClick={() => void deleteSelected()}
              style={{
                background: 'var(--accent-red, #f87171)',
                color: '#fff',
                border: 'none',
                borderRadius: 4,
                padding: '3px 10px',
                fontSize: 11,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              DELETE
            </button>
            <button
              onClick={clearSelection}
              style={{
                background: 'transparent',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '3px 10px',
                fontSize: 11,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              CANCEL
            </button>
          </div>
        </div>
      )}
      {archiveDialog && (
        <div style={{
          position: 'absolute',
          top: 44,
          left: '50%',
          transform: 'translateX(-50%)',
          zIndex: 20,
          background: 'var(--bg-panel)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: 16,
          minWidth: 300,
          maxWidth: 400,
          boxShadow: '0 8px 32px rgba(0,0,0,0.3)',
        }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: 'var(--text-primary)', marginBottom: 12 }}>
            Archive {selected_messages.length} messages
          </div>
          <div style={{ marginBottom: 10 }}>
            <label style={{ fontSize: 11, color: 'var(--text-dim)', display: 'block', marginBottom: 4 }}>Title</label>
            <input
              value={archiveTitle}
              onChange={(e) => setArchiveTitle(e.target.value)}
              style={{
                width: '100%',
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '6px 8px',
                color: 'var(--text-primary)',
                fontSize: 12,
                outline: 'none',
                boxSizing: 'border-box',
              }}
            />
          </div>
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 11, color: 'var(--text-dim)', display: 'block', marginBottom: 4 }}>Graph node type</label>
            <div style={{ display: 'flex', gap: 6 }}>
              {(['topic', 'category', 'leaf'] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setArchiveNodeType(t)}
                  style={{
                    flex: 1,
                    background: archiveNodeType === t ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                    color: archiveNodeType === t ? 'var(--bg-base)' : 'var(--text-secondary)',
                    border: '1px solid',
                    borderColor: archiveNodeType === t ? 'var(--accent-cyan)' : 'var(--border)',
                    borderRadius: 4,
                    padding: '5px 0',
                    fontSize: 11,
                    fontWeight: 600,
                    cursor: 'pointer',
                    textTransform: 'capitalize',
                  }}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>
          <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
            <button
              onClick={() => setArchiveDialog(false)}
              style={{
                background: 'transparent',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '5px 14px',
                fontSize: 11,
                cursor: 'pointer',
              }}
            >
              Cancel
            </button>
            <button
              onClick={() => void handleArchiveConfirm()}
              disabled={!archiveTitle.trim()}
              style={{
                background: archiveTitle.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: archiveTitle.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
                border: 'none',
                borderRadius: 4,
                padding: '5px 14px',
                fontSize: 11,
                fontWeight: 700,
                cursor: archiveTitle.trim() ? 'pointer' : 'default',
              }}
            >
              Archive
            </button>
          </div>
        </div>
      )}
      <ScrollContainer autoScroll={sending || undefined}>
        {messages.length === 0 && (
          <div style={{ padding: '48px 16px', textAlign: 'center', color: 'var(--text-dim)', fontSize: 12 }}>
            No messages yet. Start a conversation.
          </div>
        )}
        {messages.map((msg) => (
          <MessageItem key={msg.id} message={msg} />
        ))}
      </ScrollContainer>
    </div>
  )
}
