import { create } from 'zustand'

interface AuthState {
  token: string | null
  display_name: string | null
  avatar: string | null
  isAuthenticated: boolean
  setAuth: (token: string, display_name: string, avatar: string | null) => void
  logout: () => void
  loadFromStorage: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  display_name: null,
  avatar: null,
  isAuthenticated: false,

  setAuth: (token, display_name, avatar) => {
    localStorage.setItem('ring_token', token)
    set({ token, display_name, avatar, isAuthenticated: true })
  },

  logout: () => {
    localStorage.removeItem('ring_token')
    set({ token: null, display_name: null, avatar: null, isAuthenticated: false })
  },

  loadFromStorage: () => {
    const token = localStorage.getItem('ring_token')
    if (token) {
      set({ token, isAuthenticated: true })
    }
  },
}))
