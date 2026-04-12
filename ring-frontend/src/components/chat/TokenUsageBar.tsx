import './TokenUsageBar.css'

interface TokenUsageBarProps {
  token_count: number
  token_limit: number
}

function format_tokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

export function TokenUsageBar({ token_count, token_limit }: TokenUsageBarProps) {
  const pct = token_limit > 0 ? Math.min((token_count / token_limit) * 100, 100) : 0
  const color_class = pct >= 95 ? 'token-usage-fill-red' : pct >= 80 ? 'token-usage-fill-amber' : 'token-usage-fill-green'

  return (
    <div className="token-usage-bar">
      <div className="token-usage-track">
        <div
          className={`token-usage-fill ${color_class}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="token-usage-label">{format_tokens(token_count)} / {format_tokens(token_limit)}</span>
      {pct >= 80 && <span className="token-usage-warning">⚠</span>}
    </div>
  )
}
