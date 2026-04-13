import { useModeStore, type InteractionMode } from '../../stores/modeStore'
import { Toolbar, type ToolStatus } from '../toolbar/Toolbar'
import type { FeatureKey } from './RingSpaceLayout'
import './FooterBar.css'

const FEATURE_TABS: { key: FeatureKey; label: string; icon: string }[] = [
  { key: 'graph', label: '图谱', icon: '◉' },
  { key: 'prs', label: 'PRs', icon: '📋' },
  { key: 'members', label: '成员', icon: '👥' },
  { key: 'sessions', label: 'Session', icon: '🔍' },
]

const MODES: { key: InteractionMode; label: string }[] = [
  { key: 'daily', label: '日常' },
  { key: 'manual_archive', label: '归档' },
  { key: 'auto', label: 'Auto' },
]

interface FooterBarProps {
  tools?: ToolStatus[]
  on_tool_toggle?: (tool_name: string) => void
  show_tools?: boolean
  open_features?: Set<FeatureKey>
  on_feature_toggle?: (key: FeatureKey) => void
}

export function FooterBar({ tools = [], on_tool_toggle, show_tools = true, open_features = new Set(), on_feature_toggle }: FooterBarProps) {
  const mode = useModeStore((s) => s.mode)
  const set_mode = useModeStore((s) => s.set_mode)

  return (
    <div className="footer-bar">
      <div className="footer-bar-tabs">
        {FEATURE_TABS.map((tab) => (
          <button
            key={tab.key}
            className={`footer-tab${open_features.has(tab.key) ? ' footer-tab-active' : ''}`}
            onClick={() => on_feature_toggle?.(tab.key)}
          >
            <span className="footer-tab-icon">{tab.icon}</span>
            <span className="footer-tab-label">{tab.label}</span>
          </button>
        ))}
      </div>

      <div className="footer-bar-modes">
        {MODES.map((m) => (
          <button
            key={m.key}
            className={`footer-mode${mode === m.key ? ' footer-mode-active' : ''}`}
            onClick={() => set_mode(m.key)}
          >
            {m.label}
          </button>
        ))}
      </div>

      {show_tools && tools.length > 0 && (
        <div className="footer-bar-tools">
          <Toolbar tools={tools} on_toggle={on_tool_toggle} />
        </div>
      )}
    </div>
  )
}
