export type ParsedCommand =
  | { type: 'address'; target: string; rest: string }
  | { type: 'reference'; name: string }
  | { type: 'action'; action: string; args: string }
  | { type: 'meta'; key: string; value: string }
  | { type: 'prefs'; subcommand: 'show' | 'set'; key?: string; value?: string }
  | { type: 'skill'; subcommand: 'list' | 'install' | 'remove'; name?: string; url?: string }

export function parseCommand(input: string): ParsedCommand[] | null {
  const trimmed = input.trim()
  if (!trimmed) return null

  const commands: ParsedCommand[] = []
  const tokens = trimmed.split(/\s+/)
  let hasCommand = false
  let i = 0

  while (i < tokens.length) {
    const token = tokens[i]

    if (token.startsWith('@')) {
      hasCommand = true
      const target = token.slice(1).toLowerCase()
      const restTokens: string[] = []
      let j = i + 1
      while (j < tokens.length && !tokens[j].match(/^[@#!%]/)) {
        restTokens.push(tokens[j])
        j++
      }
      commands.push({ type: 'address', target, rest: restTokens.join(' ') })
      i = j
      continue
    }

    if (token.startsWith('#')) {
      hasCommand = true
      const name = token.slice(1)
      commands.push({ type: 'reference', name })
      i++
      continue
    }

    if (token.startsWith('!')) {
      hasCommand = true
      const action = token.slice(1).toLowerCase()
      const args = tokens.slice(i + 1).join(' ')
      commands.push({ type: 'action', action, args })
      break
    }

    if (token.startsWith('%')) {
      hasCommand = true
      const body = token.slice(1).toLowerCase()
      if (body === 'prefs') {
        const subcommand = tokens[i + 1]?.toLowerCase()
        if (subcommand === 'set' && tokens[i + 2] && tokens[i + 3]) {
          commands.push({ type: 'prefs', subcommand: 'set', key: tokens[i + 2].toLowerCase(), value: tokens.slice(i + 3).join(' ') })
        } else {
          commands.push({ type: 'prefs', subcommand: 'show' })
        }
        break
      }
      if (body === 'skill') {
        const sub = tokens[i + 1]?.toLowerCase()
        if (sub === 'install' && tokens[i + 2] && tokens[i + 3]) {
          commands.push({ type: 'skill', subcommand: 'install', name: tokens[i + 2], url: tokens.slice(i + 3).join(' ') })
        } else if (sub === 'remove' && tokens[i + 2]) {
          commands.push({ type: 'skill', subcommand: 'remove', name: tokens[i + 2] })
        } else {
          commands.push({ type: 'skill', subcommand: 'list' })
        }
        break
      }
      const nextToken = tokens[i + 1]
      commands.push({ type: 'meta', key: body, value: nextToken ?? '' })
      break
    }

    break
  }

  return hasCommand && commands.length > 0 ? commands : null
}
