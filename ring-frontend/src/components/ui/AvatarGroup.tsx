import { Avatar } from './Avatar'
import './Avatar.css'

interface AvatarGroupProps { names: string[]; max?: number; size?: 'sm' | 'md' | 'lg' }

export function AvatarGroup({ names, max = 4, size = 'sm' }: AvatarGroupProps) {
  const visible = names.slice(0, max)
  const extra = names.length - max
  return (
    <div className="avatar-group">
      {visible.map((name) => <Avatar key={name} name={name} size={size} />)}
      {extra > 0 && <span className="avatar-group-count">+{extra}</span>}
    </div>
  )
}
