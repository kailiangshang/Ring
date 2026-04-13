interface RingLogoProps {
  size?: number
}

export function RingLogo({ size = 20 }: RingLogoProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <path d="M 2 14 C 16 8, 32 22, 44 38 C 52 50, 62 52, 72 48 C 76 46, 78 44, 78 44" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" opacity="0.75"/>
      <path d="M 2 66 C 14 58, 28 46, 42 40 C 52 36, 62 40, 70 44 C 75 46, 78 44, 78 44" stroke="currentColor" strokeWidth="4" strokeLinecap="round"/>
      <path d="M 2 40 C 14 28, 30 26, 42 34 C 54 42, 60 46, 70 43 C 75 43, 78 44, 78 44" stroke="currentColor" strokeWidth="2.8" strokeLinecap="round" opacity="0.9"/>
    </svg>
  )
}
