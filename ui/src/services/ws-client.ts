type MessageHandler = (data: unknown) => void

export class WsClient {
  private ws: WebSocket | null = null
  private url: string
  private token: string
  private on_message: MessageHandler
  private on_open: () => void
  private on_close: () => void
  private reconnect_attempts = 0
  private max_reconnect_delay = 30_000
  private reconnect_timer: ReturnType<typeof setTimeout> | null = null
  private heartbeat_timer: ReturnType<typeof setInterval> | null = null
  private stopped = false
  private authenticated = false

  constructor(
    url: string,
    token: string,
    on_message: MessageHandler,
    on_open: () => void,
    on_close: () => void,
  ) {
    this.url = url
    this.token = token
    this.on_message = on_message
    this.on_open = on_open
    this.on_close = on_close
  }

  connect(): void {
    this.stopped = false
    this.authenticated = false
    this.ws = new WebSocket(this.url)

    this.ws.onopen = () => {
      this.ws!.send(JSON.stringify({ type: 'auth', token: this.token }))
    }

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data as string)
        if (!this.authenticated) {
          if (data.type === 'auth_ok') {
            this.authenticated = true
            this.reconnect_attempts = 0
            this.start_heartbeat()
            this.on_open()
          } else if (data.type === 'auth_failed') {
            this.ws?.close()
          }
          return
        }
        if (data.type === 'ping') {
          this.send({ type: 'pong', data: data.data ?? '' })
          return
        }
        this.on_message(data)
      } catch {
        void 0
      }
    }

    this.ws.onclose = () => {
      this.authenticated = false
      this.stop_heartbeat()
      this.on_close()
      if (!this.stopped) this.schedule_reconnect()
    }

    this.ws.onerror = () => {
      this.ws?.close()
    }
  }

  send(data: unknown): void {
    if (this.authenticated && this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data))
    }
  }

  disconnect(): void {
    this.stopped = true
    if (this.reconnect_timer) {
      clearTimeout(this.reconnect_timer)
      this.reconnect_timer = null
    }
    this.stop_heartbeat()
    this.ws?.close()
    this.ws = null
  }

  get connected(): boolean {
    return this.authenticated && this.ws?.readyState === WebSocket.OPEN
  }

  private schedule_reconnect(): void {
    const delay = Math.min(1000 * 2 ** this.reconnect_attempts, this.max_reconnect_delay)
    this.reconnect_attempts++
    this.reconnect_timer = setTimeout(() => this.connect(), delay)
  }

  private start_heartbeat(): void {
    this.stop_heartbeat()
    this.heartbeat_timer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }))
      }
    }, 30_000)
  }

  private stop_heartbeat(): void {
    if (this.heartbeat_timer) {
      clearInterval(this.heartbeat_timer)
      this.heartbeat_timer = null
    }
  }
}
