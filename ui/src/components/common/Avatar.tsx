interface AvatarProps {
  name: string
  avatar: string | null
  size?: number
}

export function Avatar({ name, avatar, size = 28 }: AvatarProps) {
  const isEmoji = avatar && /\p{Emoji}/u.test(avatar)
  const letter = name.charAt(0).toUpperCase()

  if (isEmoji) {
    return (
      <div
        style={{
          width: size,
          height: size,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: size * 0.6,
          borderRadius: 4,
          background: 'var(--bg-active)',
          flexShrink: 0,
        }}
      >
        {avatar}
      </div>
    )
  }

  return (
    <div
      style={{
        width: size,
        height: size,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: size * 0.45,
        fontWeight: 700,
        borderRadius: 4,
        background: 'var(--bg-active)',
        color: 'var(--accent-cyan)',
        flexShrink: 0,
      }}
    >
      {letter}
    </div>
  )
}
