export type LLMProvider = 'openai' | 'anthropic' | 'ollama'
export type InteractionMode = 'normal' | 'auto'
export type SkillPermissionMode = 'auto' | 'plan' | 'edit'

export interface LLMConfig {
  provider: LLMProvider
  model: string
  api_key_set: boolean
  base_url: string | null
}

export interface RingMode {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
}
