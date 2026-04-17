import { usePanelStore } from '../../stores/panel-store'
import { PanelWrapper } from '../panels/PanelWrapper'
import { GraphPanel } from '../panels/GraphPanel'
import { ArchivePanel } from '../panels/ArchivePanel'
import { ConfigPanel } from '../panels/ConfigPanel'
import { SessionPanel } from '../panels/SessionPanel'

const PANEL_CONTENT: Record<string, () => JSX.Element> = {
  graph: GraphPanel,
  archive: ArchivePanel,
  config: ConfigPanel,
  session: SessionPanel,
}

const PANEL_TITLES: Record<string, string> = {
  graph: 'Graph',
  archive: 'Archive',
  config: 'Config',
  session: 'Session',
}

export function PanelStack() {
  const panels = usePanelStore((s) => s.panels)
  const close = usePanelStore((s) => s.close)

  if (panels.length === 0) return null

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      {panels.map((panel, index) => {
        const Content = PANEL_CONTENT[panel.type]
        return (
          <PanelWrapper
            key={panel.type}
            title={PANEL_TITLES[panel.type]}
            depth={panel.depth}
            onClose={() => close(index)}
          >
            {Content ? <Content /> : null}
          </PanelWrapper>
        )
      })}
    </div>
  )
}
