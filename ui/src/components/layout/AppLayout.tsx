import { useEffect, lazy, Suspense } from 'react'
import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfTrigger } from '../self/SelfTrigger'
import { CreateInviteModal } from '../invite/CreateInviteModal'
import { CommandResultModal } from '../chat/CommandResultModal'
import { NotificationBell } from '../NotificationBell'
import { ExportButton } from '../chat/ExportButton'
import { useAppStore } from '../../stores/app-store'
import { useRingStore } from '../../stores/ring-store'
import { useChatStore } from '../../stores/chat-store'
import { usePanelStore } from '../../stores/panel-store'
import { useThemeStore } from '../../stores/theme-store'
import { useGraphStore } from '../../stores/graph-store'
import { GraphCanvas } from '../panels/GraphCanvas'
import { useDrag } from '../../hooks/use-drag'
import { useResize } from '../../hooks/use-resize'
import { TabItem } from '../header/TabItem'

const SelfFloat = lazy(() => import('../self/SelfFloat').then(m => ({ default: m.SelfFloat })))

function ThemeToggle() {
  const theme = useThemeStore((s) => s.theme)
  const toggleTheme = useThemeStore((s) => s.toggleTheme)

  return (
    <button
      onClick={toggleTheme}
      style={{
        background: 'none',
        border: '1px solid var(--border)',
        borderRadius: 4,
        padding: '2px 6px',
        cursor: 'pointer',
        fontSize: 14,
        lineHeight: 1,
        color: 'var(--text-secondary)',
        display: 'flex',
        alignItems: 'center',
      }}
    >
      {theme === 'dark' ? '☀️' : '🌙'}
    </button>
  )
}

function SuperRingHeader() {
  const panels = usePanelStore((s) => s.panels)
  const toggle = usePanelStore((s) => s.toggle)
  const closeAll = usePanelStore((s) => s.closeAll)

  return (
    <div style={{
      height: 38,
      background: 'var(--bg-panel)',
      borderBottom: '1px solid var(--border)',
      display: 'flex',
      alignItems: 'center',
      padding: '0 12px',
    }}>
      <span style={{
        fontSize: 13,
        fontWeight: 700,
        color: 'var(--accent-ice)',
        marginRight: 16,
        letterSpacing: '0.05em',
      }}>
        Super Ring
      </span>
      <TabItem
        label="Chat"
        active={panels.length === 0 || panels.every(p => p.type !== 'super_skills' && p.type !== 'super_settings')}
        onClick={() => {
          closeAll()
        }}
      />
      <TabItem
        label="Skills"
        active={panels.some((p) => p.type === 'super_skills')}
        onClick={() => toggle('super_skills')}
      />
      <TabItem
        label="Settings"
        active={panels.some((p) => p.type === 'super_settings')}
        onClick={() => toggle('super_settings')}
      />
      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        <ThemeToggle />
        <ExportButton />
        <NotificationBell />
      </div>
    </div>
  )
}

function GraphFloat() {
  const float_open = useGraphStore((s) => s.float_open)
  const setFloatOpen = useGraphStore((s) => s.setFloatOpen)
  const float_position = useGraphStore((s) => s.float_position)
  const float_size = useGraphStore((s) => s.float_size)
  const setFloatPosition = useGraphStore((s) => s.setFloatPosition)
  const setFloatSize = useGraphStore((s) => s.setFloatSize)
  const nodes = useGraphStore((s) => s.nodes)
  const edges = useGraphStore((s) => s.edges)
  const selected_node_id = useGraphStore((s) => s.selected_node_id)
  const collapsed_nodes = useGraphStore((s) => s.collapsed_nodes)
  const selectNode = useGraphStore((s) => s.selectNode)
  const toggleCollapse = useGraphStore((s) => s.toggleCollapse)

  const { onMouseDown: onDragDown } = useDrag(setFloatPosition, { width: float_size.w, height: float_size.h })
  const { onMouseDown: onResizeDown } = useResize(setFloatSize, { w: 400, h: 300 })

  if (!float_open) return null

  return (
    <div
      style={{
        position: 'fixed',
        left: float_position.x,
        top: float_position.y,
        width: float_size.w,
        height: float_size.h,
        background: 'var(--bg-panel)',
        border: '1px solid var(--accent-cyan)',
        borderRadius: 8,
        display: 'flex',
        flexDirection: 'column',
        zIndex: 999,
        boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
        overflow: 'hidden',
      }}
    >
      <div
        onMouseDown={onDragDown}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          borderBottom: '1px solid var(--border)',
          background: 'var(--bg-panel)',
          cursor: 'move',
          userSelect: 'none',
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-ice)', letterSpacing: '0.05em' }}>
          Graph — {nodes.length} nodes · {edges.length} edges
        </span>
        <button
          onClick={() => setFloatOpen(false)}
          style={{
            background: 'var(--bg-hover)',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '4px 12px',
            fontSize: 12,
            cursor: 'pointer',
            fontWeight: 700,
          }}
        >
          ×
        </button>
      </div>
      <div style={{ flex: 1, minHeight: 0 }}>
        <GraphCanvas
          nodes={nodes}
          edges={edges}
          selectedNodeId={selected_node_id}
          collapsedNodes={collapsed_nodes}
          onSelectNode={selectNode}
          onToggleCollapse={toggleCollapse}
          fullscreen
        />
      </div>
      <div
        onMouseDown={onResizeDown}
        style={{
          position: 'absolute',
          right: 0,
          bottom: 0,
          width: 16,
          height: 16,
          cursor: 'nwse-resize',
          zIndex: 2,
        }}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" style={{ display: 'block' }}>
          <line x1="12" y1="4" x2="4" y2="12" stroke="var(--text-dim)" strokeWidth="1" />
          <line x1="14" y1="8" x2="8" y2="14" stroke="var(--text-dim)" strokeWidth="1" />
        </svg>
      </div>
    </div>
  )
}

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)
  const fetchRings = useRingStore((s) => s.fetchRings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const loadHistory = useChatStore((s) => s.loadHistory)

  useEffect(() => {
    fetchRings()
  }, [fetchRings])

  useEffect(() => {
    loadHistory()
  }, [active_ring_id, current_context, loadHistory])

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context === 'super' ? (
          <>
            <SuperRingHeader />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
              <PanelStack />
            </div>
          </>
        ) : (
          <>
            <HeaderTabBar />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
              <PanelStack />
            </div>
          </>
        )}
      </div>
      <Suspense fallback={null}><SelfFloat /></Suspense>
      <SelfTrigger />
      <CreateInviteModal />
      <CommandResultModal />
      <GraphFloat />
    </div>
  )
}
