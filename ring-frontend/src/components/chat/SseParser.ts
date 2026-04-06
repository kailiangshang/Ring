import type { SseEvent } from '../../types'

function parse_event_lines(lines: string[]): SseEvent[] {
  const events: SseEvent[] = []

  for (const line of lines) {
    if (line === '' || line.startsWith(':')) continue
    const colon_idx = line.indexOf(':')
    if (colon_idx === -1) continue
    const field = line.slice(0, colon_idx).trim()
    const value = line.slice(colon_idx + 1).trimStart()
    if (field === 'data') {
      try {
        const parsed = JSON.parse(value) as SseEvent
        events.push(parsed)
      } catch {
        events.push({ type: 'text', content: value })
      }
    }
  }

  return events
}

export async function* parseSseStream(
  reader: ReadableStreamDefaultReader<Uint8Array>,
): AsyncGenerator<SseEvent> {
  const decoder = new TextDecoder()
  let buffer = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })

    const parts = buffer.split('\n\n')
    buffer = parts.pop() ?? ''

    for (const part of parts) {
      const lines = part.split('\n')
      for (const event of parse_event_lines(lines)) {
        yield event
      }
    }
  }

  if (buffer.trim()) {
    const lines = buffer.trim().split('\n')
    for (const event of parse_event_lines(lines)) {
      yield event
    }
  }
}
