import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useRingStore } from '../../stores/ring-store'
import {
  exportRingChat,
  exportSelfChat,
  exportSuperChat,
} from '../../services/api'

export function ExportButton() {
  const [exporting, setExporting] = useState(false)
  const context = useAppStore((s) => s.current_context)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const handleExport = async () => {
    if (exporting) return
    setExporting(true)
    try {
      if (context === 'ring' && active_ring_id) {
        await exportRingChat(active_ring_id)
      } else if (context === 'self') {
        await exportSelfChat()
      } else if (context === 'super') {
        await exportSuperChat()
      }
    } catch {
      // ignore
    }
    setExporting(false)
  }

  if (context === 'session') return null

  return (
    <button
      onClick={handleExport}
      disabled={exporting}
      title="Export chat"
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
      {exporting ? '...' : 'Export'}
    </button>
  )
}
