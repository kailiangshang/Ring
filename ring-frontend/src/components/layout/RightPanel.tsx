import './RightPanel.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

interface RightPanelProps { state: RightPanelState; on_close: () => void }

export function RightPanel({ state, on_close }: RightPanelProps) {
  const titles: Record<string, string> = { node_detail: '节点详情', diff: 'Changes', node_selector: '选择节点' }
  return (
    <div className="right-panel">
      <div className="right-panel-header">
        <h4>{state.content ? titles[state.content] || '' : ''}</h4>
        <button className="right-panel-close" onClick={on_close}>&times;</button>
      </div>
      <div className="right-panel-body">
        {state.content === 'node_detail' && <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>{state.data ? JSON.stringify(state.data) : 'No data'}</div>}
        {state.content === 'diff' && <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>Diff view（待接入）</div>}
        {state.content === 'node_selector' && <div style={{ fontSize: 'var(--font-size-sm)', color: 'var(--color-text-secondary)' }}>节点选择器（待接入）</div>}
      </div>
    </div>
  )
}
