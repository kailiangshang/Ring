import { useEffect, useCallback } from 'react'
import { MarkdownRenderer } from '../common/MarkdownRenderer'
import { useCommandResultStore } from '../../stores/command-result-store'

export function CommandResultModal() {
  const result = useCommandResultStore((s) => s.result)
  const closeCommandResult = useCommandResultStore((s) => s.closeCommandResult)

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape') closeCommandResult()
  }, [closeCommandResult])

  useEffect(() => {
    if (result) {
      document.addEventListener('keydown', handleKeyDown)
      return () => document.removeEventListener('keydown', handleKeyDown)
    }
  }, [result, handleKeyDown])

  if (!result) return null

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 9999,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) closeCommandResult()
      }}
    >
      <div style={{
        position: 'absolute',
        inset: 0,
        background: 'rgba(0, 0, 0, 0.6)',
      }} />
      <div style={{
        position: 'relative',
        width: '100%',
        maxWidth: 600,
        maxHeight: '80vh',
        margin: '0 16px',
        background: 'var(--bg-panel)',
        border: '1px solid var(--accent-cyan)',
        borderRadius: 8,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 16px',
          borderBottom: '1px solid var(--border)',
          flexShrink: 0,
        }}>
          <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-cyan)', letterSpacing: '0.05em' }}>
            {result.title}
          </span>
          <button
            onClick={closeCommandResult}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--text-muted)',
              fontSize: 16,
              cursor: 'pointer',
              padding: '0 4px',
              lineHeight: 1,
            }}
          >
            ✕
          </button>
        </div>
        <div style={{
          padding: '16px',
          overflow: 'auto',
          color: 'var(--text-primary)',
          fontSize: 13,
          lineHeight: 1.6,
        }}>
          <MarkdownRenderer content={result.content} />
        </div>
      </div>
    </div>
  )
}
