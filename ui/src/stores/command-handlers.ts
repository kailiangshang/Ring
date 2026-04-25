import type { ChatMessage } from '../types/chat'
import { getPreferences, updatePreferences, listSkills, installSkill, removeSkill } from '../services/api'
import { useAppStore } from './app-store'

const SCOPE_LABELS: Record<string, string> = {
  super: 'Super',
  ring: 'Ring',
  session: 'Session',
}

export function buildHelpContent(): string {
  type CmdInfo = { prefix: string; cmd: string; desc: string; scopes: string[] }

  const slashCmds: CmdInfo[] = [
    { prefix: '/', cmd: 'graph', desc: 'Open graph panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'archive', desc: 'Open archive panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'config', desc: 'Open config panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'session [create/close/start/summarize]', desc: 'Session operations', scopes: ['ring', 'session'] },
    { prefix: '/', cmd: 'new <name>', desc: 'Create new ring', scopes: ['ring'] },
    { prefix: '/', cmd: 'save', desc: 'Archive conversation', scopes: ['ring', 'session'] },
    { prefix: '/', cmd: 'node [add/link]', desc: 'Graph node operations', scopes: ['ring'] },
    { prefix: '/', cmd: 'mode [auto/normal]', desc: 'Set interaction mode', scopes: ['ring'] },
    { prefix: '/', cmd: 'prefs [set key value]', desc: 'Show/set preferences', scopes: ['super', 'ring'] },
    { prefix: '/', cmd: 'skill [list/install/remove]', desc: 'Manage skills', scopes: ['super', 'ring'] },
    { prefix: '/', cmd: 'members', desc: 'Show members', scopes: ['ring'] },
    { prefix: '/', cmd: 'invite [open/audit]', desc: 'Create invite', scopes: ['ring'] },
    { prefix: '/', cmd: 'help [command]', desc: 'Show help', scopes: ['super', 'ring', 'session'] },
  ]

  const atCmds: CmdInfo[] = [
    { prefix: '@', cmd: 'self [message]', desc: 'Talk to Self', scopes: ['super', 'ring', 'session'] },
    { prefix: '@', cmd: 'ring [message]', desc: 'Talk to Ring AI', scopes: ['ring'] },
    { prefix: '@', cmd: 'super [message]', desc: 'Talk to Super Ring', scopes: ['super', 'ring'] },
    { prefix: '@', cmd: 'node <name>', desc: 'Reference graph node', scopes: ['ring'] },
  ]

  const currentContext = useAppStore.getState().current_context

  const renderTable = (cmds: CmdInfo[]) => {
    const header = '| Command | Description | Scope |'
    const sep = '|---------|-------------|-------|'
    const rows = cmds.map(c => {
      const scopeStr = c.scopes.map(s => SCOPE_LABELS[s] ?? s).join(', ')
      const marker = c.scopes.includes(currentContext) ? '' : ' 🔒'
      return `| ${c.prefix}${c.cmd}${marker} | ${c.desc} | ${scopeStr} |`
    })
    return [header, sep, ...rows].join('\n')
  }

  return `## Commands\n\n> Scope: **Super** = Super Ring only · **Ring** = Group Ring · **Session** = Active session · 🔒 = not available in current view\n\n### Slash Commands (/ prefix)\n${renderTable(slashCmds)}\n\n### Addressing (@ prefix)\n${renderTable(atCmds)}`
}

export function getCommandHelp(command: string): string {
  const helpMap: Record<string, string> = {
    graph: '### /graph\n\nOpen the graph panel to view and edit the knowledge graph.\n\n**Usage:** `/graph`',
    archive: '### /archive\n\nOpen the archive panel to view archived conversations.\n\n**Usage:** `/archive`',
    config: '### /config\n\nOpen the configuration panel.\n\n**Usage:** `/config`',
    session: '### /session\n\nSession operations.\n\n**Usage:**\n- `/session` - Open session panel\n- `/session create <title>` - Create new session\n- `/session close` - Close current session\n- `/session start` - Start discussion\n- `/session summarize` - AI summary',
    new: '### /new\n\nCreate a new Ring.\n\n**Usage:** `/new <ring-name>`',
    save: '### /save\n\nArchive the current conversation.\n\n**Usage:** `/save [optional-title]`',
    node: '### /node\n\nGraph node operations.\n\n**Usage:**\n- `/node add <name>` - Add new node\n- `/node link <from> <to>` - Link two nodes',
    mode: '### /mode\n\nSet interaction mode.\n\n**Usage:** `/mode [auto/normal]`',
    prefs: '### /prefs\n\nManage preferences.\n\n**Usage:**\n- `/prefs` - Show preferences\n- `/prefs set <key> <value>` - Set preference',
    skill: '### /skill\n\nManage skills.\n\n**Usage:**\n- `/skill list` - List skills\n- `/skill install <name> <url>` - Install skill\n- `/skill remove <name>` - Remove skill',
    members: '### /members\n\nShow member list.\n\n**Usage:** `/members`',
    invite: '### /invite\n\nCreate invitation tokens.\n\n**Usage:**\n- `/invite open` - Open invitation\n- `/invite audit` - Audit invitation',
    help: '### /help\n\nShow help information.\n\n**Usage:**\n- `/help` - Show all commands\n- `/help <command>` - Show specific command help',
    cross_ring_query: '### /cross-ring-query\n\nQuery across all your Rings.\n\n**Usage:** `/cross-ring-query <your question>`',
    cross_ring_analysis: '### /cross-ring-analysis\n\nAnalyze multiple Rings.\n\n**Usage:** `/cross-ring-analysis <compare|merge|summary> <ring1,ring2,...> [question]`',
  }

  return helpMap[command] || `No help available for command: ${command}`
}

const PREFS_KEY_MAP: Record<string, { section: string; key: string }> = {
  language: { section: '语言', key: 'default' },
  provider: { section: 'LLM', key: 'default_provider' },
  style: { section: '输出格式', key: 'style' },
  mode: { section: '默认模式', key: 'mode' },
}

export async function handlePrefsShow(addMessage: (msg: ChatMessage) => void) {
  try {
    const { content, is_custom } = await getPreferences()
    const label = is_custom ? '当前偏好设置（自定义）：' : '当前偏好设置（默认）：'
    addMessage({
      id: `sys-prefs-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `${label}\n\`\`\`\n${content}\n\`\`\``,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: 'Failed to load preferences.',
      created_at: new Date().toISOString(),
    })
  }
}

export async function handlePrefsSet(key: string, value: string, addMessage: (msg: ChatMessage) => void) {
  const mapping = PREFS_KEY_MAP[key]
  if (!mapping) {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Unknown preference key "${key}". Supported keys: ${Object.keys(PREFS_KEY_MAP).join(', ')}. For other changes, ask Super Ring.`,
      created_at: new Date().toISOString(),
    })
    return
  }

  try {
    const { content } = await getPreferences()
    const lines = content.split('\n')
    let inSection = false
    let found = false
    const updated = lines.map(line => {
      if (line.trim() === `## ${mapping.section}`) {
        inSection = true
        return line
      }
      if (inSection && line.trim().startsWith(`- ${mapping.key}:`)) {
        found = true
        return `- ${mapping.key}: ${value}`
      }
      if (line.startsWith('## ') && inSection) {
        inSection = false
      }
      return line
    }).join('\n')

    if (!found) {
      addMessage({
        id: `sys-prefs-err-${crypto.randomUUID()}`,
        role: 'system',
        sender_name: 'SYSTEM',
        content: `Could not find preference "${key}" in current settings. Please use Super Ring to modify.`,
        created_at: new Date().toISOString(),
      })
      return
    }

    await updatePreferences(updated)
    addMessage({
      id: `sys-prefs-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Preference updated: ${key} = ${value}`,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Failed to update preference "${key}".`,
      created_at: new Date().toISOString(),
    })
  }
}

export async function handleSkillList(addMessage: (msg: ChatMessage) => void) {
  try {
    const { skills } = await listSkills()
    if (skills.length === 0) {
      addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'No skills installed.', created_at: new Date().toISOString() })
      return
    }
    const lines = skills.map(s => {
      const tag = s.source === 'builtin' ? '[built-in]' : '[user]'
      return `- **${s.name}** ${tag}: ${s.description}`
    })
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `## Skills\n\n${lines.join('\n')}`, created_at: new Date().toISOString() })
  } catch {
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'Failed to load skills.', created_at: new Date().toISOString() })
  }
}

export async function handleSkillInstall(name: string, url: string, addMessage: (msg: ChatMessage) => void) {
  try {
    const result = await installSkill(name, url)
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: result.ok ? `Skill "${result.name}" installed: ${result.description}` : 'Install failed', created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill install failed: ${msg}`, created_at: new Date().toISOString() })
  }
}

export async function handleSkillRemove(name: string, addMessage: (msg: ChatMessage) => void) {
  try {
    await removeSkill(name)
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill "${name}" removed.`, created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Failed to remove skill: ${msg}`, created_at: new Date().toISOString() })
  }
}
