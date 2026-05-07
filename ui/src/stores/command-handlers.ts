import type { ChatMessage } from '../types/chat'
import { getPreferences, updatePreferences, listSkills, installSkill, removeSkill, compactChat, compactSelfChat, compactSuperChat } from '../services/api'
import { useAppStore } from './app-store'
import { useRingStore } from './ring-store'

export function buildHelpContent(): string {
  type CmdGroup = { title: string; cmds: { usage: string; desc: string; scopes: string[] }[] }

  const groups: CmdGroup[] = [
    {
      title: '图谱',
      cmds: [
        { usage: '/node add <label>', desc: '创建节点', scopes: ['ring'] },
        { usage: '/node link <from> <to> [relation]', desc: '创建关联', scopes: ['ring'] },
        { usage: '/graph', desc: '打开图谱面板', scopes: ['ring'] },
      ],
    },
    {
      title: '归档',
      cmds: [
        { usage: '/save', desc: '归档对话', scopes: ['ring', 'session'] },
        { usage: '/archive', desc: '打开归档面板', scopes: ['ring'] },
      ],
    },
    {
      title: 'Session',
      cmds: [
        { usage: '/session create <title>', desc: '创建 Session', scopes: ['ring'] },
        { usage: '/session start', desc: '开始讨论', scopes: ['ring'] },
        { usage: '/session summarize', desc: 'AI 总结', scopes: ['ring'] },
        { usage: '/session close', desc: '关闭 Session', scopes: ['ring'] },
      ],
    },
    {
      title: '跨 Ring',
      cmds: [
        { usage: '/cross-ring-query <query>', desc: '跨 Ring 搜索', scopes: ['super', 'ring'] },
        { usage: '/cross-ring-analysis <rings> <type>', desc: '跨 Ring 分析', scopes: ['super'] },
      ],
    },
    {
      title: '配置',
      cmds: [
        { usage: '/config', desc: '查看配置', scopes: ['ring'] },
        { usage: '/prefs', desc: '偏好设置', scopes: ['super', 'ring'] },
        { usage: '/mode [auto/normal]', desc: '交互模式', scopes: ['ring'] },
      ],
    },
    {
      title: '成员',
      cmds: [
        { usage: '/members', desc: '查看成员', scopes: ['ring'] },
        { usage: '/invite open', desc: '创建邀请', scopes: ['ring'] },
        { usage: '/invite audit', desc: '审计邀请', scopes: ['ring'] },
      ],
    },
    {
      title: 'Self',
      cmds: [
        { usage: '/skill [list/install/remove]', desc: '管理 Skills', scopes: ['super', 'ring'] },
        { usage: '@self [message]', desc: '对话 Self', scopes: ['super', 'ring', 'session'] },
        { usage: '@ring [message]', desc: '对话 Ring AI', scopes: ['ring'] },
        { usage: '@super [message]', desc: '对话 Super Ring', scopes: ['super', 'ring'] },
      ],
    },
    {
      title: '其他',
      cmds: [
        { usage: '/compact', desc: '压缩聊天历史', scopes: ['super', 'ring'] },
        { usage: '/new <name>', desc: '创建 Ring', scopes: ['ring'] },
        { usage: '/help [command]', desc: '帮助', scopes: ['super', 'ring', 'session'] },
      ],
    },
  ]

  const currentContext = useAppStore.getState().current_context

  const lines: string[] = ['## 命令列表\n']

  for (const group of groups) {
    const visibleCmds = group.cmds.filter((c) => c.scopes.includes(currentContext))
    if (visibleCmds.length === 0) continue

    lines.push(`### ${group.title}`)
    for (const cmd of visibleCmds) {
      lines.push(`- \`${cmd.usage}\` — ${cmd.desc}`)
    }
    lines.push('')
  }

  return lines.join('\n')
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
    compact: '### /compact\n\nCompress chat history using AI summarization.\n\n**Usage:** `/compact`',
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

export async function handlePrefsShow(
  addMessage: (msg: ChatMessage) => void,
  showResult?: (title: string, content: string) => void,
) {
  try {
    const { content, is_custom } = await getPreferences()
    const label = is_custom ? '当前偏好设置（自定义）：' : '当前偏好设置（默认）：'
    const md = `${label}\n\`\`\`\n${content}\n\`\`\``
    if (showResult) {
      showResult('/prefs', md)
    } else {
      addMessage({
        id: `sys-prefs-${crypto.randomUUID()}`,
        role: 'system',
        sender_name: 'SYSTEM',
        content: md,
        created_at: new Date().toISOString(),
      })
    }
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

export async function handleSkillList(
  addMessage: (msg: ChatMessage) => void,
  showResult?: (title: string, content: string) => void,
) {
  try {
    const { skills } = await listSkills()
    if (skills.length === 0) {
      if (showResult) {
        showResult('/skill list', 'No skills installed.')
      } else {
        addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'No skills installed.', created_at: new Date().toISOString() })
      }
      return
    }
    const lines = skills.map(s => {
      const tag = s.source === 'builtin' ? '[built-in]' : '[user]'
      return `- **${s.name}** ${tag}: ${s.description}`
    })
    const md = `## Skills\n\n${lines.join('\n')}`
    if (showResult) {
      showResult('/skill list', md)
    } else {
      addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: md, created_at: new Date().toISOString() })
    }
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

export async function handleCompact(addMessage: (msg: ChatMessage) => void) {
  addMessage({ id: `sys-compact-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'Compressing history...', created_at: new Date().toISOString() })

  try {
    const context = useAppStore.getState().current_context
    const ringId = useRingStore.getState().active_ring_id

    let result: { summary: string; removed_count: number }
    if (context === 'ring' && ringId) {
      result = await compactChat(ringId)
    } else if (context === 'super') {
      result = await compactSuperChat()
    } else {
      result = await compactSelfChat()
    }

    if (result.removed_count > 0) {
      addMessage({ id: `sys-compact-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `History compressed. ${result.removed_count} messages summarized.`, created_at: new Date().toISOString() })
    } else {
      addMessage({ id: `sys-compact-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: result.summary, created_at: new Date().toISOString() })
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-compact-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Compact failed: ${msg}`, created_at: new Date().toISOString() })
  }
}
