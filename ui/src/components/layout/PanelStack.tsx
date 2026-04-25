import { type ComponentType, Suspense, lazy } from 'react'
import { usePanelStore } from '../../stores/panel-store'
import { PanelWrapper } from '../panels/PanelWrapper'

const GraphPanel = lazy(() => import('../panels/GraphPanel').then(m => ({ default: m.GraphPanel })))
const ArchivePanel = lazy(() => import('../panels/ArchivePanel').then(m => ({ default: m.ArchivePanel })))
const ConfigPanel = lazy(() => import('../panels/ConfigPanel').then(m => ({ default: m.ConfigPanel })))
const SessionPanel = lazy(() => import('../panels/SessionPanel').then(m => ({ default: m.SessionPanel })))
const SuperSkillsPanel = lazy(() => import('../panels/SuperSkillsPanel').then(m => ({ default: m.SuperSkillsPanel })))
const SuperSettingsPanel = lazy(() => import('../panels/SuperSettingsPanel').then(m => ({ default: m.SuperSettingsPanel })))
const BlueprintPanel = lazy(() => import('../panels/BlueprintPanel').then(m => ({ default: m.BlueprintPanel })))

const PANEL_CONTENT: Record<string, ComponentType> = {
  graph: GraphPanel,
  archive: ArchivePanel,
  config: ConfigPanel,
  session: SessionPanel,
  super_skills: SuperSkillsPanel,
  super_settings: SuperSettingsPanel,
  blueprint: BlueprintPanel,
}

const PANEL_TITLES: Record<string, string> = {
  graph: 'Graph',
  archive: 'Archive',
  config: 'Config',
  session: 'Session',
  super_skills: 'Skills',
  super_settings: 'Settings',
  blueprint: 'Blueprint',
}

function PanelFallback() {
  return (
    <div style={{ padding: 20, color: 'var(--text-dim)', fontSize: 11 }}>
      Loading...
    </div>
  )
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
            <Suspense fallback={<PanelFallback />}>
              {Content ? <Content /> : null}
            </Suspense>
          </PanelWrapper>
        )
      })}
    </div>
  )
}
