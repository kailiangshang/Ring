import { useRef, useEffect } from 'react'
import { useAutocompleteStore } from './autocomplete-store'

export function CommandAutocomplete({ onSelect }: { onSelect: (val: string) => void }) {
  const visible = useAutocompleteStore((s) => s.visible)
  const matches = useAutocompleteStore((s) => s.matches)
  const selectedIndex = useAutocompleteStore((s) => s.selectedIndex)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!containerRef.current || selectedIndex < 0) return
    const items = containerRef.current.children
    if (items[selectedIndex]) {
      items[selectedIndex].scrollIntoView({ block: 'nearest' })
    }
  }, [selectedIndex])

  if (!visible || matches.length === 0) return null

  return (
    <div
      ref={containerRef}
      style={{
        position: 'absolute',
        bottom: '100%',
        left: 0,
        right: 0,
        background: 'var(--bg-panel)',
        border: '1px solid var(--border)',
        borderRadius: '4px 4px 0 0',
        maxHeight: 200,
        overflow: 'auto',
        zIndex: 100,
      }}
    >
      {matches.map((cmd, i) => (
        <div
          key={`${cmd.trigger}${cmd.cmd}${cmd.subcommand || ''}`}
          onMouseDown={(e) => {
            e.preventDefault()
            const val = cmd.subcommand
              ? `${cmd.trigger}${cmd.cmd} ${cmd.subcommand} `
              : `${cmd.trigger}${cmd.cmd} `
            onSelect(val)
            useAutocompleteStore.getState().hide()
          }}
          onMouseEnter={() => useAutocompleteStore.setState({ selectedIndex: i })}
          style={{
            padding: '6px 12px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            background: i === selectedIndex ? 'var(--bg-hover)' : 'transparent',
            fontSize: 12,
          }}
        >
          <span style={{ color: 'var(--accent-cyan)', fontWeight: 700, minWidth: 70 }}>
            {cmd.subcommand ? `${cmd.trigger}${cmd.cmd} ${cmd.subcommand}` : `${cmd.trigger}${cmd.cmd}`}
          </span>
          <span style={{ color: 'var(--text-muted)' }}>{cmd.desc}</span>
        </div>
      ))}
    </div>
  )
}
