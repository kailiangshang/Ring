import { describe, it, expect } from 'vitest'
import { parseSseStream } from './SseParser'
import type { SseEvent } from '../../types'

function make_stream(chunks: string[]): ReadableStream<Uint8Array> {
  return new ReadableStream({
    start(controller) {
      for (const chunk of chunks) {
        controller.enqueue(new TextEncoder().encode(chunk))
      }
      controller.close()
    },
  })
}

async function collect(gen: AsyncGenerator<SseEvent>): Promise<SseEvent[]> {
  const results: SseEvent[] = []
  for await (const event of gen) {
    results.push(event)
  }
  return results
}

describe('SseParser', () => {
  it('parses single event', async () => {
    const stream = make_stream(['data: {"type":"text","content":"hello"}\n\n'])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([{ type: 'text', content: 'hello' }])
  })

  it('parses multiple events in one chunk', async () => {
    const stream = make_stream([
      'data: {"type":"text","content":"hi"}\n\ndata: {"type":"done"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([
      { type: 'text', content: 'hi' },
      { type: 'done' },
    ])
  })

  it('handles partial chunks across boundaries', async () => {
    const stream = make_stream([
      'data: {"type":"text","conte',
      'nt":"world"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([{ type: 'text', content: 'world' }])
  })

  it('handles event split across boundary', async () => {
    const stream = make_stream([
      'data: {"type":"text","content":"a"}\n\n',
      'data: {"type":"tool_call","tool_name":"search"}\n\ndata: {"type":"don',
      'e"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toHaveLength(3)
    expect(events[0]).toEqual({ type: 'text', content: 'a' })
    expect(events[1]).toEqual({ type: 'tool_call', tool_name: 'search' })
    expect(events[2]).toEqual({ type: 'done' })
  })

  it('ignores comments', async () => {
    const stream = make_stream([
      ': this is a comment\ndata: {"type":"done"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([{ type: 'done' }])
  })

  it('yields remaining buffered data at end', async () => {
    const stream = make_stream([
      'data: {"type":"text","content":"tail"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([{ type: 'text', content: 'tail' }])
  })

  it('handles error events', async () => {
    const stream = make_stream([
      'data: {"type":"error","message":"something broke"}\n\n',
    ])
    const reader = stream.getReader()
    const events = await collect(parseSseStream(reader))
    expect(events).toEqual([{ type: 'error', message: 'something broke' }])
  })
})
