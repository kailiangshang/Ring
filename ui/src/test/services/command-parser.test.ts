import { describe, it, expect } from 'vitest'
import { parseCommand } from '../../services/command-parser'

describe('parseCommand', () => {
  it('returns null for plain text', () => {
    expect(parseCommand('hello world')).toBeNull()
  })

  it('parses @self', () => {
    const result = parseCommand('@self hello')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'self', rest: 'hello' })
  })

  it('parses @ring', () => {
    const result = parseCommand('@ring 分析一下')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'ring', rest: '分析一下' })
  })

  it('parses @super', () => {
    const result = parseCommand('@super 总结')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'address', target: 'super', rest: '总结' })
  })

  it('parses !graph', () => {
    const result = parseCommand('!graph')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'graph', args: '' })
  })

  it('parses !save with args', () => {
    const result = parseCommand('!save some content')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'save', args: 'some content' })
  })

  it('parses !auto as toggle', () => {
    const result = parseCommand('!auto')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'auto', args: '' })
  })

  it('parses %skill plan', () => {
    const result = parseCommand('%skill plan')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'meta', key: 'skill', value: 'plan' })
  })

  it('parses %mode auto', () => {
    const result = parseCommand('%mode auto')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'meta', key: 'mode', value: 'auto' })
  })

  it('parses #nodename', () => {
    const result = parseCommand('#竞品分析')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'reference', name: '竞品分析' })
  })

  it('parses multiple commands in one input', () => {
    const result = parseCommand('@ring #竞品分析 帮我看看这个节点')
    expect(result).toHaveLength(2)
    expect(result![0]).toEqual({ type: 'address', target: 'ring', rest: '' })
    expect(result![1]).toEqual({ type: 'reference', name: '竞品分析' })
  })

  it('parses !new with args for ring creation', () => {
    const result = parseCommand('!new 竞品分析组')
    expect(result).toHaveLength(1)
    expect(result![0]).toEqual({ type: 'action', action: 'new', args: '竞品分析组' })
  })

  it('returns null for empty input', () => {
    expect(parseCommand('')).toBeNull()
    expect(parseCommand('   ')).toBeNull()
  })
})
