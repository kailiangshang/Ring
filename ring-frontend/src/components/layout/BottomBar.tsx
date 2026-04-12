import { useModeStore, type InteractionMode } from '../../stores/modeStore'
import { Toolbar, type ToolStatus } from '../toolbar/Toolbar'
import './BottomBar.css'

const MODES: { key: InteractionMode; label: string }[] = [
  { key: 'daily', label: '日常' },
  { key: 'manual_archive', label: '手动归档' },
  { key: 'auto', label: 'Auto' },
]

interface BottomBarProps {
  tools?: ToolStatus[]
  on_tool_toggle?: (tool_name: string) => void
  show_tools?: boolean
}

export function BottomBar({ tools = [], on_tool_toggle, show_tools = true }: BottomBarProps) {
  const mode = useModeStore((s) => s.mode)
  const set_mode = useModeStore((s) => s.set_mode)

  return (
    <div className="bottom-bar">
      <div className="bottom-bar-left">
        {MODES.map((m) => (
          <button
            key={m.key}
            className={`mode-btn${mode === m.key ? ' mode-btn-active' : ''}`}
            onClick={() => set_mode(m.key)}
          >
            {m.label}
          </button>
        ))}
      </div>
      {show_tools && tools.length > 0 && (
        <div className="bottom-bar-right">
          <Toolbar tools={tools} on_toggle={on_tool_toggle} />
        </div>
      )}
    </div>
  )
}
