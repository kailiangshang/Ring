import { create } from 'zustand'

interface ThemeState {
  theme: 'dark' | 'light'
  toggleTheme: () => void
}

function applyTheme(theme: 'dark' | 'light') {
  document.documentElement.setAttribute('data-theme', theme)
}

function getInitialTheme(): 'dark' | 'light' {
  const saved = localStorage.getItem('ring_theme')
  if (saved === 'light' || saved === 'dark') {
    applyTheme(saved)
    return saved
  }
  applyTheme('dark')
  return 'dark'
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: getInitialTheme(),

  toggleTheme: () =>
    set((state) => {
      const next = state.theme === 'dark' ? 'light' : 'dark'
      localStorage.setItem('ring_theme', next)
      applyTheme(next)
      return { theme: next }
    }),
}))
