import { describe, it, expect } from 'vitest'
import { parseCommand } from '../../services/command-parser'

describe('parseCommand', () => {
  it('returns null for plain text', () => {
    expect(parseCommand('hello world')).toBeNull()
  })

  it('returns null for old prefixes', () => {
    expect(parseCommand('!graph')).toBeNull()
    expect(parseCommand('%prefs')).toBeNull()
    expect(parseCommand('#node')).toBeNull()
  })

  it('parses /graph', () => {
    const result = parseCommand('/graph')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'graph', args: '' })
  })

  it('parses /archive', () => {
    const result = parseCommand('/archive')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'archive', args: '' })
  })

  it('parses /session create with title', () => {
    const result = parseCommand('/session create My Session')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'session', subcommand: 'create', args: 'My Session' })
  })

  it('parses /session close', () => {
    const result = parseCommand('/session close')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'session', subcommand: 'close', args: '' })
  })

  it('parses /node add with name', () => {
    const result = parseCommand('/node add 竞品分析')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'node', subcommand: 'add', args: '竞品分析' })
  })

  it('parses /skill list', () => {
    const result = parseCommand('/skill list')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'skill', subcommand: 'list', args: '' })
  })

  it('parses /mode auto', () => {
    const result = parseCommand('/mode auto')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'mode', args: 'auto' })
  })

  it('parses /help', () => {
    const result = parseCommand('/help')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'help' })
  })

  it('parses /help session', () => {
    const result = parseCommand('/help session')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'help', command: 'session' })
  })

  it('parses @self with message', () => {
    const result = parseCommand('@self hello')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'self', rest: 'hello' })
  })

  it('parses @ring with message', () => {
    const result = parseCommand('@ring 分析一下')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'ring', rest: '分析一下' })
  })

  it('parses @super with message', () => {
    const result = parseCommand('@super 总结')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'super', rest: '总结' })
  })

  it('parses @node with name', () => {
    const result = parseCommand('@node 竞品分析')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'node', rest: '竞品分析' })
  })

  it('parses @self without message', () => {
    const result = parseCommand('@self')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'self', rest: '' })
  })

  it('returns null for empty input', () => {
    expect(parseCommand('')).toBeNull()
    expect(parseCommand('   ')).toBeNull()
  })
})
