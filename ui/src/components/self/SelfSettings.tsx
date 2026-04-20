import { useEffect, useState } from 'react'
import { api } from '../../services/api'

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '5px 8px',
  color: 'var(--text-primary)',
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 8,
  marginTop: 2,
  resize: 'vertical' as const,
}

const sectionStyle: React.CSSProperties = {
  marginBottom: 12,
  padding: '6px 8px',
  background: 'var(--bg-base)',
  borderRadius: 3,
  border: '1px solid var(--border)',
}

const labelStyle: React.CSSProperties = {
  fontSize: 10,
  fontWeight: 700,
  color: 'var(--accent-amber)',
  letterSpacing: '0.05em',
  marginBottom: 4,
  display: 'block',
}

const smallBtn: React.CSSProperties = {
  background: 'var(--accent-amber)',
  color: 'var(--bg-base)',
  border: 'none',
  borderRadius: 3,
  padding: '4px 10px',
  fontSize: 10,
  fontWeight: 700,
  cursor: 'pointer',
  marginBottom: 8,
}

export function SelfSettings() {
  const [identity, setIdentity] = useState('')
  const [style, setStyle] = useState('')
  const [tone, setTone] = useState('friendly')
  const [proactivity, setProactivity] = useState(true)
  const [suggestions, setSuggestions] = useState(true)
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null)

  useEffect(() => {
    api.get<{ content: string; exists: boolean }>('/self/identity')
      .then((res) => { if (res.exists) setIdentity(res.content) })
      .catch(() => {})
    api.get<{ content: string; exists: boolean }>('/self/style')
      .then((res) => { if (res.exists) setStyle(res.content) })
      .catch(() => {})
  }, [])

  const saveIdentity = async () => {
    try {
      await api.put('/self/identity', { content: identity })
      setMsg({ ok: true, text: 'Identity saved' })
    } catch { setMsg({ ok: false, text: 'Failed' }) }
  }

  const saveStyle = async () => {
    try {
      await api.put('/self/style', { content: style })
      setMsg({ ok: true, text: 'Style saved' })
    } catch { setMsg({ ok: false, text: 'Failed' }) }
  }

  return (
    <div style={{ padding: 8, fontSize: 12, overflow: 'auto', flex: 1 }}>
      <div style={sectionStyle}>
        <span style={labelStyle}>Personality</span>
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 2 }}>Tone</div>
          <div style={{ display: 'flex', gap: 3 }}>
            {['friendly', 'professional', 'playful', 'minimal'].map((t) => (
              <button
                key={t}
                onClick={() => setTone(t)}
                style={{
                  background: tone === t ? 'var(--accent-amber)' : 'var(--bg-hover)',
                  color: tone === t ? 'var(--bg-base)' : 'var(--text-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  cursor: 'pointer',
                  fontWeight: tone === t ? 700 : 400,
                }}
              >
                {t}
              </button>
            ))}
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
          <button
            onClick={() => setProactivity(!proactivity)}
            style={{
              width: 28,
              height: 14,
              borderRadius: 7,
              border: 'none',
              cursor: 'pointer',
              background: proactivity ? 'var(--accent-amber)' : 'var(--bg-hover)',
              position: 'relative',
            }}
          >
            <span style={{
              position: 'absolute',
              top: 1,
              left: proactivity ? 14 : 1,
              width: 12,
              height: 12,
              borderRadius: '50%',
              background: 'var(--bg-base)',
              transition: 'left 0.15s',
            }} />
          </button>
          <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>Proactivity</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <button
            onClick={() => setSuggestions(!suggestions)}
            style={{
              width: 28,
              height: 14,
              borderRadius: 7,
              border: 'none',
              cursor: 'pointer',
              background: suggestions ? 'var(--accent-amber)' : 'var(--bg-hover)',
              position: 'relative',
            }}
          >
            <span style={{
              position: 'absolute',
              top: 1,
              left: suggestions ? 14 : 1,
              width: 12,
              height: 12,
              borderRadius: '50%',
              background: 'var(--bg-base)',
              transition: 'left 0.15s',
            }} />
          </button>
          <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>Suggestions</span>
        </div>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Identity</span>
        <textarea
          value={identity}
          onChange={(e) => setIdentity(e.target.value)}
          placeholder="Define who Self is..."
          style={{ ...inputStyle, minHeight: 50 }}
        />
        <button onClick={saveIdentity} style={smallBtn}>SAVE</button>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Style</span>
        <textarea
          value={style}
          onChange={(e) => setStyle(e.target.value)}
          placeholder="Define conversation style..."
          style={{ ...inputStyle, minHeight: 50 }}
        />
        <button onClick={saveStyle} style={smallBtn}>SAVE</button>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Data</span>
        <div style={{ display: 'flex', gap: 6 }}>
          <button
            onClick={() => {
              const data = { identity, style, tone, proactivity, suggestions }
              const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
              const url = URL.createObjectURL(blob)
              const a = document.createElement('a')
              a.href = url
              a.download = 'self-data.json'
              a.click()
              URL.revokeObjectURL(url)
            }}
            style={{ ...smallBtn, background: 'var(--bg-hover)', color: 'var(--text-primary)', border: '1px solid var(--border)' }}
          >
            EXPORT
          </button>
          <button
            onClick={() => {
              if (window.confirm('Reset all Self data? This cannot be undone.')) {
                api.put('/self/identity', { content: '' })
                api.put('/self/style', { content: '' })
                setIdentity('')
                setStyle('')
                setMsg({ ok: true, text: 'Data reset' })
              }
            }}
            style={{ ...smallBtn, background: 'transparent', color: 'var(--accent-amber)', border: '1px solid var(--accent-amber)' }}
          >
            RESET
          </button>
        </div>
      </div>

      {msg && (
        <div style={{
          fontSize: 10,
          padding: '4px 8px',
          borderRadius: 3,
          background: msg.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
          color: msg.ok ? 'var(--accent-green)' : '#ef4444',
          border: `1px solid ${msg.ok ? 'var(--accent-green)' : '#ef4444'}`,
        }}>
          {msg.text}
        </div>
      )}
    </div>
  )
}
