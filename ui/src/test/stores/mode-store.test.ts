import { describe, it, expect, beforeEach } from 'vitest'
import { useModeStore } from '../../stores/mode-store'

describe('modeStore', () => {
  beforeEach(() => {
    useModeStore.getState().reset()
  })

  it('defaults to normal/plan', () => {
    const s = useModeStore.getState()
    expect(s.interaction_mode).toBe('normal')
    expect(s.skill_permission_mode).toBe('plan')
  })

  it('toggles auto mode', () => {
    useModeStore.getState().toggleAuto()
    expect(useModeStore.getState().interaction_mode).toBe('auto')
    useModeStore.getState().toggleAuto()
    expect(useModeStore.getState().interaction_mode).toBe('normal')
  })

  it('sets skill permission mode', () => {
    useModeStore.getState().setSkillMode('edit')
    expect(useModeStore.getState().skill_permission_mode).toBe('edit')
  })
})
