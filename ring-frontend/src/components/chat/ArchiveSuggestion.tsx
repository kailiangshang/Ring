import { Button } from '../ui/Button'
import './ArchiveSuggestion.css'

export function ArchiveSuggestion({ data, on_accept, on_dismiss }: {
  data: unknown
  on_accept: () => void
  on_dismiss: () => void
}) {
  const suggestion = data as { reason?: string; suggested_title?: string }
  return (
    <div className="archive-suggestion">
      <div className="archive-suggestion-text">{suggestion.reason || 'AI suggests archiving this conversation'}</div>
      <div className="archive-suggestion-actions">
        <Button size="sm" onClick={on_accept}>Accept</Button>
        <Button size="sm" variant="secondary" onClick={on_dismiss}>Dismiss</Button>
      </div>
    </div>
  )
}
