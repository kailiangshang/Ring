import './EmptyState.css'
import { Button } from './Button'

interface EmptyStateProps {
  icon?: string
  title: string
  description?: string
  action_label?: string
  on_action?: () => void
}

export function EmptyState({ icon, title, description, action_label, on_action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      {icon && <div className="empty-state-icon">{icon}</div>}
      <div className="empty-state-title">{title}</div>
      {description && <div className="empty-state-desc">{description}</div>}
      {action_label && on_action && <Button onClick={on_action}>{action_label}</Button>}
    </div>
  )
}
