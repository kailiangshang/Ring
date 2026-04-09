import './Tabs.css'

interface Tab { key: string; label: string }

interface TabsProps { tabs: Tab[]; active_key: string; on_change: (key: string) => void }

export function Tabs({ tabs, active_key, on_change }: TabsProps) {
  return (
    <div className="tabs">
      {tabs.map((tab) => (
        <button key={tab.key} className={`tab-item${tab.key === active_key ? ' tab-active' : ''}`} onClick={() => on_change(tab.key)}>
          {tab.label}
        </button>
      ))}
    </div>
  )
}
