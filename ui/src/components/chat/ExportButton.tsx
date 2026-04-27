import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useRingStore } from '../../stores/ring-store'
import {
  exportRingChat,
  exportChatPdf,
  exportSelfChat,
  exportSuperChat,
} from '../../services/api'

export function ExportButton() {
  const [exporting, setExporting] = useState(false)
  const [result, setResult] = useState<{ ok: boolean; text: string } | null>(null)
  const context = useAppStore((s) => s.current_context)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const handleExport = async (format: 'md' | 'pdf') => {
    if (exporting) return
    setExporting(true)
    setResult(null)
    try {
      if (context === 'ring' && active_ring_id) {
        if (format === 'pdf') {
          await exportChatPdf(active_ring_id)
        } else {
          await exportRingChat(active_ring_id)
        }
      } else if (context === 'self') {
        await exportSelfChat()
      } else if (context === 'super') {
        await exportSuperChat()
      }
      setResult({ ok: true, text: 'Exported!' })
    } catch {
      setResult({ ok: false, text: 'Export failed' })
    }
    setExporting(false)
    setTimeout(() => setResult(null), 2000)
  }

  if (context === 'session') return null

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
      <button
        onClick={() => handleExport('md')}
        disabled={exporting}
        title="Export chat as Markdown"
        style={{
          background: 'none',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '4px 8px',
          color: 'var(--text-secondary)',
          fontSize: 11,
          cursor: 'pointer',
          opacity: exporting ? 0.5 : 1,
        }}
      >
        {exporting ? 'Exporting...' : 'Export'}
      </button>
      {context === 'ring' && active_ring_id && (
        <button
          onClick={() => handleExport('pdf')}
          disabled={exporting}
          title="Export chat as PDF"
          style={{
            background: 'none',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '4px 8px',
            color: 'var(--text-secondary)',
            fontSize: 11,
            cursor: 'pointer',
            opacity: exporting ? 0.5 : 1,
          }}
        >
          PDF
        </button>
      )}
      {result && (
        <span style={{ fontSize: 10, color: result.ok ? 'var(--accent-green)' : 'var(--accent-amber)' }}>
          {result.text}
        </span>
      )}
    </div>
  )
}
