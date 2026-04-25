import { useState, useEffect } from 'react'
import { api } from '../../services/api'

interface PromptEntry {
  module: string
  name: string
  content: string
}

export function PromptsPanel() {
  const [prompts, setPrompts] = useState<PromptEntry[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [filter, setFilter] = useState('')

  useEffect(() => {
    api.get<PromptEntry[]>('/prompts').then(setPrompts).catch(() => {})
  }, [])

  const filtered = prompts.filter(
    (p) =>
      p.module.toLowerCase().includes(filter.toLowerCase()) ||
      p.name.toLowerCase().includes(filter.toLowerCase()),
  )

  const active = prompts.find((p) => `${p.module}.${p.name}` === selected)

  return (
    <div style={{ display: 'flex', height: '100%', fontSize: 12 }}>
      <div
        style={{
          width: 180,
          minWidth: 180,
          borderRight: '1px solid var(--border)',
          overflow: 'auto',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div style={{ padding: '8px' }}>
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter..."
            style={{
              width: '100%',
              background: 'var(--bg-input)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '4px 6px',
              color: 'var(--text-primary)',
              fontSize: 11,
              fontFamily: 'inherit',
              outline: 'none',
            }}
          />
        </div>
        {filtered.map((p) => {
          const key = `${p.module}.${p.name}`
          return (
            <div
              key={key}
              onClick={() => setSelected(key)}
              style={{
                padding: '6px 10px',
                cursor: 'pointer',
                background: selected === key ? 'var(--bg-hover)' : 'transparent',
                borderLeft:
                  selected === key
                    ? '2px solid var(--accent-cyan)'
                    : '2px solid transparent',
              }}
            >
              <div
                style={{ color: 'var(--accent-cyan)', fontWeight: 700, fontSize: 10 }}
              >
                {p.module}
              </div>
              <div style={{ color: 'var(--text-secondary)', fontSize: 11 }}>
                {p.name}
              </div>
            </div>
          )
        })}
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>
        {active ? (
          <>
            <div
              style={{
                marginBottom: 8,
                display: 'flex',
                gap: 8,
                alignItems: 'center',
              }}
            >
              <span style={{ color: 'var(--accent-cyan)', fontWeight: 700 }}>
                {active.module}
              </span>
              <span style={{ color: 'var(--text-dim)' }}>·</span>
              <span style={{ color: 'var(--text-secondary)' }}>
                {active.name}
              </span>
            </div>
            <pre
              style={{
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                fontFamily: 'Cascadia Code, monospace',
                fontSize: 11,
                lineHeight: 1.6,
                color: 'var(--text-primary)',
                margin: 0,
              }}
            >
              {active.content}
            </pre>
          </>
        ) : (
          <div
            style={{
              color: 'var(--text-dim)',
              textAlign: 'center',
              paddingTop: 40,
            }}
          >
            Select a prompt to view
          </div>
        )}
      </div>
    </div>
  )
}
