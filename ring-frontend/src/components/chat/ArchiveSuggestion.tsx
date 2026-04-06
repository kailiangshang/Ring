export function ArchiveSuggestion({ data, on_accept, on_dismiss }: {
  data: unknown
  on_accept: () => void
  on_dismiss: () => void
}) {
  const suggestion = data as { reason?: string; suggested_title?: string }
  return (
    <div style={{
      padding: '10px 14px',
      borderRadius: 8,
      backgroundColor: '#fff3e0',
      borderLeft: '3px solid #ff9800',
      marginBottom: 8,
    }}>
      <div style={{ marginBottom: 8 }}>{suggestion.reason || 'AI suggests archiving this conversation'}</div>
      <div style={{ display: 'flex', gap: 8 }}>
        <button onClick={on_accept} style={{ padding: '4px 12px', cursor: 'pointer' }}>Accept</button>
        <button onClick={on_dismiss} style={{ padding: '4px 12px', cursor: 'pointer' }}>Dismiss</button>
      </div>
    </div>
  )
}
