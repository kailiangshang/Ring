export interface SseMessageStart {
  message_id: string
  role: string
}

export interface SseDelta {
  content: string
}

export interface SseMessageEnd {
  message_id: string
  usage: { prompt_tokens: number; completion_tokens: number }
}

export interface SseError {
  error: string
}

export interface SseCallbacks {
  onStart: (data: SseMessageStart) => void
  onDelta: (data: SseDelta) => void
  onEnd: (data: SseMessageEnd) => void
  onError: (data: SseError) => void
}

export function streamChat(url: string, body: unknown, callbacks: SseCallbacks): AbortController {
  const controller = new AbortController()

  const token = localStorage.getItem('ring_token')

  fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { 'X-Ring-Token': token } : {}),
    },
    body: JSON.stringify(body),
    signal: controller.signal,
  })
    .then(async (res) => {
      if (!res.ok) {
        const err = await res.json().catch(() => ({}))
        callbacks.onError({ error: err?.error?.message ?? res.statusText })
        return
      }

      const reader = res.body?.getReader()
      if (!reader) {
        callbacks.onError({ error: 'No response body' })
        return
      }

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() ?? ''

        let currentEvent = ''
        for (const line of lines) {
          if (line.startsWith('event: ')) {
            currentEvent = line.slice(7).trim()
          } else if (line.startsWith('data: ')) {
            const data = line.slice(6)
            try {
              const parsed = JSON.parse(data)
              switch (currentEvent) {
                case 'message_start':
                  callbacks.onStart(parsed)
                  break
                case 'delta':
                  callbacks.onDelta(parsed)
                  break
                case 'message_end':
                  callbacks.onEnd(parsed)
                  break
                case 'error':
                  callbacks.onError(parsed)
                  break
              }
            } catch {
              // skip malformed JSON
            }
            currentEvent = ''
          }
        }
      }
    })
    .catch((e) => {
      if (e.name !== 'AbortError') {
        callbacks.onError({ error: e.message })
      }
    })

  return controller
}
