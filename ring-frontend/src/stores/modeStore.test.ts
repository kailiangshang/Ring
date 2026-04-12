import { describe, it, expect, beforeEach } from 'vitest'
import { useModeStore } from './modeStore'

describe('modeStore', () => {
  beforeEach(() => {
    useModeStore.setState({ mode: 'daily' })
  })

  it('defaults to daily mode', () => {
    expect(useModeStore.getState().mode).toBe('daily')
  })

  it('switches to manual_archive', () => {
    useModeStore.getState().set_mode('manual_archive')
    expect(useModeStore.getState().mode).toBe('manual_archive')
  })

  it('switches to auto', () => {
    useModeStore.getState().set_mode('auto')
    expect(useModeStore.getState().mode).toBe('auto')
  })

  it('switches back to daily', () => {
    useModeStore.getState().set_mode('auto')
    useModeStore.getState().set_mode('daily')
    expect(useModeStore.getState().mode).toBe('daily')
  })
})
