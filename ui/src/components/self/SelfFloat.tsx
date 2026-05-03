import { useEffect, useState } from 'react'
import { useSelfStore } from '../../stores/self-store'
import { useDrag } from '../../hooks/use-drag'
import { SelfChat } from './SelfChat'
import { SelfMemory } from './SelfMemory'
import { SelfSettings } from './SelfSettings'
import { SelfActivity } from './SelfActivity'
import { api } from '../../services/api'

const TABS = [
  { key: 'chat' as const, label: 'Chat' },
  { key: 'memory' as const, label: 'Memory' },
  { key: 'activity' as const, label: '活动' },
  { key: 'settings' as const, label: 'Settings' },
]

const TAB_CONTENT = {
  chat: SelfChat,
  memory: SelfMemory,
  activity: SelfActivity,
  settings: SelfSettings,
}

export function SelfFloat() {
  const { open, position, setPosition, active_tab, setTab, setOpen } = useSelfStore()
  const { onMouseDown } = useDrag(setPosition, { width: 340, height: 380 })
  const [greeting, setGreeting] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      api.get<{ greeting: string | null; first_today: boolean }>('/self/greeting')
        .then((res) => {
          if (res.greeting) {
            setGreeting(res.greeting)
            setTimeout(() => setGreeting(null), 10000)
          }
        })
        .catch(() => {})
    }
  }, [open])

  if (!open) return null

  const Content = TAB_CONTENT[active_tab]

  return (
    <div
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        width: 340,
        height: 380,
        background: 'var(--bg-panel)',
        border: '1px solid var(--accent-amber)',
        borderRadius: 8,
        display: 'flex',
        flexDirection: 'column',
        zIndex: 999,
        boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
      }}
    >
      {greeting && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '6px 12px',
            background: 'rgba(245,158,11,0.08)',
            borderBottom: '1px solid rgba(245,158,11,0.15)',
            fontSize: 11,
            color: 'var(--accent-amber)',
          }}
        >
          <span>{greeting}</span>
          <button
            onClick={() => setGreeting(null)}
            style={{ background: 'none', border: 'none', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 12, padding: '0 4px' }}
          >
            ×
          </button>
        </div>
      )}
      <div
        onMouseDown={onMouseDown}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          borderBottom: '1px solid var(--border)',
          cursor: 'move',
          userSelect: 'none',
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-amber)' }}>
          Self
        </span>
        <button
          onClick={() => setOpen(false)}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            fontSize: 14,
          }}
        >
          ×
        </button>
      </div>

      <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
        {TABS.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setTab(tab.key)}
            style={{
              flex: 1,
              background: 'none',
              border: 'none',
              borderBottom: active_tab === tab.key ? '2px solid var(--accent-amber)' : '2px solid transparent',
              color: active_tab === tab.key ? 'var(--accent-amber)' : 'var(--text-muted)',
              fontSize: 11,
              padding: '6px 0',
              cursor: 'pointer',
              fontWeight: active_tab === tab.key ? 700 : 400,
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <Content />
    </div>
  )
}
