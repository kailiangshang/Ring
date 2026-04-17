import { describe, it, expect, beforeEach } from 'vitest'
import { usePanelStore } from '../../stores/panel-store'

describe('panelStore', () => {
  beforeEach(() => {
    usePanelStore.getState().closeAll()
  })

  it('opens a panel', () => {
    usePanelStore.getState().open('graph')
    expect(usePanelStore.getState().panels).toEqual([{ type: 'graph', depth: 1 }])
  })

  it('stacks panels', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    const state = usePanelStore.getState().panels
    expect(state).toHaveLength(2)
    expect(state[0].depth).toBe(1)
    expect(state[1].depth).toBe(2)
  })

  it('closes a single panel by index', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    usePanelStore.getState().close(0)
    expect(usePanelStore.getState().panels).toHaveLength(1)
    expect(usePanelStore.getState().panels[0].type).toBe('archive')
    expect(usePanelStore.getState().panels[0].depth).toBe(1)
  })

  it('toggles panel', () => {
    usePanelStore.getState().toggle('graph')
    expect(usePanelStore.getState().panels).toHaveLength(1)
    usePanelStore.getState().toggle('graph')
    expect(usePanelStore.getState().panels).toHaveLength(0)
  })

  it('closeAll removes all panels', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    usePanelStore.getState().closeAll()
    expect(usePanelStore.getState().panels).toHaveLength(0)
  })
})
