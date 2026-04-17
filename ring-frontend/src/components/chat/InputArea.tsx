import { useChatStore } from '../../stores/chat-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'

export function InputArea() {
  const { input, setInput } = useChatStore()

  return (
    <div>
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
          onChange={(e) => setInput(e.target.value)}
          placeholder="message / command..."
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
        <button
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
      </div>
      <CommandHints />
    </div>
  )
}
