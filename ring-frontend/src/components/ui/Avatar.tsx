import './Avatar.css'

const COLORS = [
  'var(--color-node-concept)',
  'var(--color-node-document)',
  'var(--color-node-category)',
  'var(--color-node-event)',
  'var(--color-node-avatar-rose)',
  'var(--color-node-person)',
]

function hash_color(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) { hash = name.charCodeAt(i) + ((hash << 5) - hash) }
  return COLORS[Math.abs(hash) % COLORS.length]
}

interface AvatarProps { name: string; size?: 'sm' | 'md' | 'lg' }

export function Avatar({ name, size = 'md' }: AvatarProps) {
  return (
    <div className={`avatar avatar-${size}`} style={{ background: hash_color(name) }} title={name}>
      {name.charAt(0).toUpperCase()}
    </div>
  )
}
