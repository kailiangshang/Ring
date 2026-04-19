export interface SkillInfo {
  name: string
  description: string
  source: 'builtin' | 'user'
  installed_at: string | null
}

export interface SkillDetail {
  name: string
  description: string
  source: string
  content: string
}

export interface InstallResult {
  ok: boolean
  name: string
  description: string
}
