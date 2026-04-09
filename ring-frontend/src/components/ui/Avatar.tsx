import './Avatar.css'

const COLORS = ['#2563EB', '#16A34A', '#D97706', '#7C3AED', '#DB2777', '#0891B2']

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
