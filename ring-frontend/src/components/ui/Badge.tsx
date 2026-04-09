import './Badge.css'

type BadgeVariant = 'accent' | 'success' | 'danger' | 'warning' | 'neutral'

const STATUS_MAP: Record<string, BadgeVariant> = {
  active: 'accent', opened: 'accent', merged: 'success', success: 'success',
  closed: 'danger', error: 'danger', warning: 'warning',
  creator: 'warning', admin: 'accent', member: 'success', readonly: 'neutral',
}

interface BadgeProps {
  variant?: BadgeVariant
  status?: string
  children: React.ReactNode
}

export function Badge({ variant, status, children }: BadgeProps) {
  const v = variant || (status ? STATUS_MAP[status] || 'neutral' : 'neutral')
  return <span className={`badge badge-${v}`}>{children}</span>
}
