import { Button } from '../ui/Button'
import './ArchiveSuggestion.css'

export interface ArchiveSuggestionData {
  reason?: string
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  action_preview?: string
  target_node_id?: string
}

export function ArchiveSuggestion({ data, on_accept, on_dismiss }: {
  data: unknown
  on_accept: (suggestion: ArchiveSuggestionData) => void
  on_dismiss: () => void
}) {
  const suggestion = data as ArchiveSuggestionData
  return (
    <div className="archive-suggestion">
      <div className="archive-suggestion-text">{suggestion.reason || 'AI 建议归档此对话内容'}</div>
      {suggestion.suggested_title && (
        <div className="archive-suggestion-title">📄 {suggestion.suggested_title}</div>
      )}
      {suggestion.suggested_parent && (
        <div className="archive-suggestion-parent">📂 {suggestion.suggested_parent.label}</div>
      )}
      {suggestion.action_preview && (
        <div className="archive-suggestion-preview">{suggestion.action_preview}</div>
      )}
      <div className="archive-suggestion-actions">
        <Button size="sm" onClick={() => on_accept(suggestion)}>归档</Button>
        <Button size="sm" variant="secondary" onClick={on_dismiss}>跳过</Button>
      </div>
    </div>
  )
}
