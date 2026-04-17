export function SessionIndicator() {
  return (
    <div
      style={{
        marginLeft: 28,
        padding: '4px 8px',
        fontSize: 11,
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: 6,
      }}
    >
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: '50%',
          background: 'var(--accent-green)',
        }}
      />
      1 active session
    </div>
  )
}
