export type ParsedCommand =
  | { type: 'address'; target: string; rest: string }
  | { type: 'action'; action: string; subcommand?: string; args: string }
  | { type: 'help'; command?: string }

export function parseCommand(input: string): ParsedCommand[] | null {
  const trimmed = input.trim()
  if (!trimmed) return null

  // Old prefixes are no longer supported
  if (trimmed.startsWith('!') || trimmed.startsWith('%') || trimmed.startsWith('#')) {
    return null
  }

  if (trimmed.startsWith('/')) {
    return parseSlashCommand(trimmed)
  }

  if (trimmed.startsWith('@')) {
    return parseAddressCommand(trimmed)
  }

  return null
}

function parseSlashCommand(input: string): ParsedCommand[] | null {
  const tokens = input.slice(1).split(/\s+/)
  const command = tokens[0]?.toLowerCase()
  
  if (!command) return null

  if (command === 'help') {
    const targetCommand = tokens[1]?.toLowerCase()
    return [{ type: 'help', command: targetCommand }]
  }

  // Commands with subcommands
  const subcommandCommands = ['session', 'skill', 'node', 'invite', 'prefs']
  if (subcommandCommands.includes(command) && tokens[1]) {
    return [{
      type: 'action',
      action: command,
      subcommand: tokens[1].toLowerCase(),
      args: tokens.slice(2).join(' ')
    }]
  }

  return [{
    type: 'action',
    action: command,
    args: tokens.slice(1).join(' ')
  }]
}

function parseAddressCommand(input: string): ParsedCommand[] | null {
  const tokens = input.slice(1).split(/\s+/)
  const target = tokens[0]?.toLowerCase()
  
  if (!target) return null

  const rest = tokens.slice(1).join(' ')
  return [{ type: 'address', target, rest }]
}
