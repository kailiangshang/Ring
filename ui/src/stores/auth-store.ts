import { create } from 'zustand'

interface AuthState {
  token: string | null
  display_name: string | null
  avatar: string | null
  isAuthenticated: boolean
  token_expired: boolean
  setAuth: (token: string, display_name: string, avatar: string | null) => void
  logout: () => void
  loadFromStorage: () => void
  setTokenExpired: (expired: boolean) => void
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  display_name: null,
  avatar: null,
  isAuthenticated: false,
  token_expired: false,

  setAuth: (token, display_name, avatar) => {
    localStorage.setItem('ring_token', token)
    set({ token, display_name, avatar, isAuthenticated: true, token_expired: false })
  },

  logout: () => {
    localStorage.removeItem('ring_token')
    set({ token: null, display_name: null, avatar: null, isAuthenticated: false, token_expired: false })
  },

  loadFromStorage: () => {
    const token = localStorage.getItem('ring_token')
    if (token) {
      set({ token, isAuthenticated: true })
    }
  },

  setTokenExpired: (expired: boolean) => {
    if (expired) {
      localStorage.removeItem('ring_token')
      set({ token: null, display_name: null, avatar: null, isAuthenticated: false, token_expired: true })
    } else {
      set({ token_expired: false })
    }
  },
}))
