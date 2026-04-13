import type { ExportType, ExportTypeInfo } from '../../types'
import { EXPORT_TYPES } from './ExportPanel'
import './ExportTypeList.css'

interface ExportTypeListProps {
  selected: ExportType | null
  on_select: (type: ExportType) => void
}

function group_label(group: ExportTypeInfo['group']): string {
  switch (group) {
    case 'graph': return '图谱'
    case 'content': return '内容'
    case 'ai': return 'AI'
    case 'backup': return '备份'
  }
}

const GROUP_ORDER: ExportTypeInfo['group'][] = ['graph', 'content', 'ai', 'backup']

export function ExportTypeList({ selected, on_select }: ExportTypeListProps) {
  const groups = GROUP_ORDER.map((g) => ({
    label: group_label(g),
    items: EXPORT_TYPES.filter((t) => t.group === g),
  }))

  return (
    <div className="export-type-list">
      {groups.map((group) => (
        <div key={group.label} className="export-type-group">
          <div className="export-type-group-label">{group.label}</div>
          {group.items.map((item) => (
            <button
              key={item.type}
              className={`export-type-item${selected === item.type ? ' export-type-selected' : ''}`}
              onClick={() => on_select(item.type)}
            >
              <span className="export-type-icon">{item.icon}</span>
              <div className="export-type-info">
                <div className="export-type-label">{item.label}</div>
                <div className="export-type-desc">{item.description}</div>
              </div>
            </button>
          ))}
        </div>
      ))}
    </div>
  )
}
