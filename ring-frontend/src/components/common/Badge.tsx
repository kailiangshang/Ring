interface BadgeProps {
  count: number
}

export function Badge({ count }: BadgeProps) {
  if (count <= 0) return null
  return (
    <span
      style={{
        background: 'var(--accent-cyan)',
        color: 'var(--bg-base)',
        fontSize: 10,
        fontWeight: 700,
        padding: '0 5px',
        borderRadius: 8,
        minWidth: 16,
        height: 16,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {count > 99 ? '99+' : count}
    </span>
  )
}
