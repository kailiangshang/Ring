import { useState, useRef } from 'react'
import { useChatStore } from '../../stores/chat-store'
import { useCommandHistoryStore } from '../../stores/command-history-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import { useSelfStore } from '../../stores/self-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'
import { CommandAutocomplete, useAutocompleteStore } from './CommandAutocomplete'
import { uploadFile, parseFile } from '../../services/api'

export function InputArea() {
  const { input, setInput, send, sending, stopStreaming, addMessage } = useChatStore()
  const ac = useAutocompleteStore()
  const [historyIndex, setHistoryIndex] = useState(-1)
  const inputRef = useRef<HTMLTextAreaElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)

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
      const target = e.currentTarget as HTMLTextAreaElement
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
      const target = e.currentTarget as HTMLTextAreaElement
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

    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault()
      if (input.trim()) {
        useCommandHistoryStore.getState().add(input.trim())
      }
      setHistoryIndex(-1)
      handleSend()
    }
  }

  const handleChange = (val: string) => {
    setInput(val)
    ac.update(val)
    if (historyIndex !== -1) {
      setHistoryIndex(-1)
    }
    requestAnimationFrame(() => {
      const el = inputRef.current
      if (el) {
        el.style.height = 'auto'
        const line_height = parseFloat(getComputedStyle(el).lineHeight) || 20
        const max_height = line_height * 6
        el.style.height = Math.min(el.scrollHeight, max_height) + 'px'
      }
    })
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
      useRingStore.getState().selectRing(null)
      useAppStore.getState().setContext('super')
      setTimeout(() => inputRef.current?.focus(), 50)
    } else {
      setInput(val)
      setTimeout(() => inputRef.current?.focus(), 0)
    }
  }

  const [parsedFiles, setParsedFiles] = useState<Array<{filename: string, content: string}>>([])

  const handleFileUpload = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    setUploading(true)

    const current_context = useAppStore.getState().current_context
    const active_ring_id = useRingStore.getState().active_ring_id
    const active_session_id = useAppStore.getState().active_session_id

    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      const parsingMsgId = `sys-${crypto.randomUUID()}`
      addMessage({
        id: parsingMsgId,
        role: 'system',
        sender_name: 'SYSTEM',
        content: `📄 Parsing ${file.name}...`,
        created_at: new Date().toISOString(),
      })
      try {
        if (current_context === 'session' && active_ring_id && active_session_id) {
          const endpoint = `/rings/${active_ring_id}/sessions/${active_session_id}/material-prep/upload`
          await uploadFile(endpoint, file)
          addMessage({
            id: `sys-${crypto.randomUUID()}`,
            role: 'system',
            sender_name: 'SYSTEM',
            content: `✅ Uploaded to session: ${file.name}`,
            created_at: new Date().toISOString(),
          })
        } else {
          const parsed = await parseFile(file)
          setParsedFiles(prev => [...prev, { filename: parsed.filename, content: parsed.content }])
          addMessage({
            id: `sys-${crypto.randomUUID()}`,
            role: 'system',
            sender_name: 'SYSTEM',
            content: `✅ Parsed ${file.name} (${(parsed.content.length / 1024).toFixed(1)} KB). Add your message and Ctrl+Enter to send.`,
            created_at: new Date().toISOString(),
          })
        }
      } catch (e: any) {
        const errorMsg = typeof e?.message === 'string' ? e.message : String(e)
        console.error('upload failed:', errorMsg)
        addMessage({
          id: `sys-${crypto.randomUUID()}`,
          role: 'system',
          sender_name: 'SYSTEM',
          content: `❌ Failed to parse ${file.name}: ${errorMsg || 'unknown error'}`,
          created_at: new Date().toISOString(),
        })
      }
    }
    setUploading(false)
  }

  const handleSend = () => {
    if (!input.trim() && parsedFiles.length === 0) return
    
    // Build full content (files + user input) for AI, but UI shows clean version
    if (parsedFiles.length > 0) {
      const filesContent = parsedFiles.map(f => 
        `📎 File: ${f.filename}\n---\n${f.content}`
      ).join('\n\n')
      const fullContent = input.trim() 
        ? `${input}\n\n${filesContent}`
        : filesContent
      // Send full content to AI, UI will show clean version
      send(fullContent)
      setParsedFiles([])
    } else {
      send()
    }
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    handleFileUpload(e.dataTransfer.files)
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  const handlePaste = (e: React.ClipboardEvent) => {
    if (e.clipboardData.files.length > 0) {
      e.preventDefault()
      handleFileUpload(e.clipboardData.files)
    }
  }

  return (
    <div style={{ position: 'relative' }} onDrop={handleDrop} onDragOver={handleDragOver}>
      {parsedFiles.length > 0 && (
        <div style={{ padding: '4px 12px', display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          {parsedFiles.map((f, i) => (
            <span
              key={i}
              style={{
                fontSize: 11,
                color: 'var(--accent-cyan)',
                background: 'var(--bg-hover)',
                padding: '2px 8px',
                borderRadius: 4,
                display: 'flex',
                alignItems: 'center',
                gap: 4,
              }}
            >
              📎 {f.filename}
              <button
                onClick={() => setParsedFiles(prev => prev.filter((_, idx) => idx !== i))}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--text-dim)',
                  cursor: 'pointer',
                  fontSize: 10,
                  padding: 0,
                }}
              >
                ✕
              </button>
            </span>
          ))}
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
          ref={fileInputRef}
          type="file"
          multiple
          accept=".txt,.md,.csv,.json,.py,.js,.ts,.tsx,.rs,.go,.java,.yaml,.yml,.xml,.html,.css,.toml,.sh,.sql,.log,.pdf"
          style={{ display: 'none' }}
          onChange={(e) => handleFileUpload(e.target.files)}
        />
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          style={{
            background: 'none',
            border: 'none',
            color: uploading ? 'var(--text-dim)' : 'var(--text-secondary)',
            cursor: uploading ? 'default' : 'pointer',
            fontSize: 16,
            padding: '4px 4px',
            lineHeight: 1,
          }}
          title="Upload file"
        >
          {uploading ? '⏳' : '📎'}
        </button>
        <textarea
          ref={inputRef}
          rows={1}
          value={input}
          onChange={(e) => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          onPaste={handlePaste}
          placeholder="Type / for commands, @ to address... (Ctrl+Enter to send)"
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
            resize: 'none',
            minHeight: 36,
            maxHeight: 144,
            lineHeight: '20px',
            overflowY: 'auto',
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
            onClick={handleSend}
            disabled={!input.trim() && parsedFiles.length === 0}
            style={{
              background: (input.trim() || parsedFiles.length > 0) ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: (input.trim() || parsedFiles.length > 0) ? 'var(--bg-base)' : 'var(--text-dim)',
              border: 'none',
              borderRadius: 4,
              padding: '8px 16px',
              fontSize: 12,
              fontWeight: 700,
              cursor: (input.trim() || parsedFiles.length > 0) ? 'pointer' : 'default',
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
