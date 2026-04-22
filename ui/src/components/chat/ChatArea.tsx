import { useEffect, useState } from 'react'
import { MessageList } from './MessageList'
import { InputArea } from './InputArea'
import { getTokenCount } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'

const TOKEN_THRESHOLD = 100_000

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

export function ChatArea() {
  const [tokens, setTokens] = useState(0)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const context = useAppStore((s) => s.current_context)

  useEffect(() => {
    if (context !== 'ring' || !active_ring_id) {
      setTokens(0)
      return
    }
    getTokenCount(active_ring_id)
      .then((res) => setTokens(res.total_tokens))
      .catch(() => setTokens(0))
  }, [active_ring_id, context])

  const pct = tokens / TOKEN_THRESHOLD
  const color = pct >= 1 ? 'var(--accent-red)' : pct >= 0.8 ? 'var(--accent-amber)' : 'var(--text-dim)'

  return (
    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-end',
          padding: '4px 12px',
          fontSize: 11,
          color,
          borderBottom: '1px solid var(--border)',
          gap: 6,
        }}
      >
        <span>{formatTokens(tokens)} / {formatTokens(TOKEN_THRESHOLD)}</span>
        {pct >= 0.8 && pct < 1 && <span>⚠️ 接近上限</span>}
        {pct >= 1 && <span>🔴 已达上限</span>}
      </div>
      <MessageList />
      <InputArea />
    </div>
  )
}
