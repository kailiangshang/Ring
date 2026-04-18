import { create } from 'zustand'
import { WsClient } from '../services/ws-client'

type WsMessageHandler = (data: unknown) => void

interface WsState {
  connected: boolean
  connecting: boolean
  client: WsClient | null
  handlers: WsMessageHandler[]
  addHandler: (handler: WsMessageHandler) => void
  removeHandler: (handler: WsMessageHandler) => void
  connect: () => void
  disconnect: () => void
  send: (data: unknown) => void
}

export const useWsStore = create<WsState>((set, get) => ({
  connected: false,
  connecting: false,
  client: null,
  handlers: [],

  addHandler: (handler) => {
    set((s) => ({ handlers: [...s.handlers, handler] }))
  },

  removeHandler: (handler) => {
    set((s) => ({ handlers: s.handlers.filter((h) => h !== handler) }))
  },

  connect: () => {
    const { client } = get()
    if (client) return

    const token = localStorage.getItem('ring_token')
    if (!token) return

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = `${protocol}//${window.location.host}/api/ws?token=${encodeURIComponent(token)}`

    const ws_client = new WsClient(
      url,
      (data) => {
        const { handlers } = get()
        for (const handler of handlers) {
          handler(data)
        }
      },
      () => set({ connected: true, connecting: false }),
      () => set({ connected: false, connecting: false }),
    )

    set({ client: ws_client, connecting: true })
    ws_client.connect()
  },

  disconnect: () => {
    get().client?.disconnect()
    set({ client: null, connected: false, connecting: false })
  },

  send: (data) => {
    get().client?.send(data)
  },
}))
