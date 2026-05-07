import { useEffect, useState } from 'react'
import {
  api,
  getSelfPersonality,
  updateSelfPersonality,
  getSelfPrivacy,
  updateSelfPrivacy,
  exportSelfData,
  resetSelfData,
} from '../../services/api'
import { ConfirmModal } from '../../components/common/ConfirmModal'

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
  const [privacyLevel, setPrivacyLevel] = useState('standard')
  const [shareMetrics, setShareMetrics] = useState(false)
  const [allowProactive, setAllowProactive] = useState(true)
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null)
  const [confirmDialog, setConfirmDialog] = useState<{ title: string; message: string; action: () => void; variant?: 'danger' | 'default' } | null>(null)

  useEffect(() => {
    api.get<{ content: string; exists: boolean }>('/self/identity')
      .then((res) => { if (res.exists) setIdentity(res.content) })
      .catch(() => {})
    api.get<{ content: string; exists: boolean }>('/self/style')
      .then((res) => { if (res.exists) setStyle(res.content) })
      .catch(() => {})
    getSelfPersonality()
      .then((res) => {
        setTone(res.tone)
        setProactivity(res.proactivity)
        setSuggestions(res.suggestions)
      })
      .catch(() => {})
    getSelfPrivacy()
      .then((res) => {
        setPrivacyLevel(res.level)
        setShareMetrics(res.share_metrics)
        setAllowProactive(res.allow_proactive)
      })
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

  const savePersonality = async () => {
    try {
      await updateSelfPersonality({ tone, proactivity, suggestions })
      setMsg({ ok: true, text: 'Personality saved' })
    } catch { setMsg({ ok: false, text: 'Failed' }) }
  }

  const savePrivacy = async () => {
    try {
      await updateSelfPrivacy({
        level: privacyLevel,
        share_metrics: shareMetrics,
        allow_proactive: allowProactive,
      })
      setMsg({ ok: true, text: 'Privacy saved' })
    } catch { setMsg({ ok: false, text: 'Failed' }) }
  }

  const handleExport = async () => {
    try {
      const data = await exportSelfData()
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'self-data.json'
      a.click()
      URL.revokeObjectURL(url)
      setMsg({ ok: true, text: 'Data exported' })
    } catch {
      setMsg({ ok: false, text: 'Export failed' })
    }
  }

  const handleReset = () => {
    setConfirmDialog({
      title: 'Reset Self Data',
      message: 'Reset all Self data? This cannot be undone.',
      variant: 'danger',
      action: async () => {
        try {
          await resetSelfData()
          setIdentity('')
          setStyle('')
          setTone('friendly')
          setProactivity(true)
          setSuggestions(true)
          setPrivacyLevel('standard')
          setShareMetrics(false)
          setAllowProactive(true)
          setMsg({ ok: true, text: 'Data reset' })
        } catch {
          setMsg({ ok: false, text: 'Reset failed' })
        }
      },
    })
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
        <button onClick={savePersonality} style={{ ...smallBtn, marginTop: 8 }}>SAVE PERSONALITY</button>
      </div>

      <div style={sectionStyle}>
        <span style={labelStyle}>Privacy</span>
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 2 }}>Level</div>
          <div style={{ display: 'flex', gap: 3 }}>
            {['standard', 'strict', 'minimal'].map((l) => (
              <button
                key={l}
                onClick={() => setPrivacyLevel(l)}
                style={{
                  background: privacyLevel === l ? 'var(--accent-amber)' : 'var(--bg-hover)',
                  color: privacyLevel === l ? 'var(--bg-base)' : 'var(--text-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  cursor: 'pointer',
                  fontWeight: privacyLevel === l ? 700 : 400,
                }}
              >
                {l}
              </button>
            ))}
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
          <button
            onClick={() => setShareMetrics(!shareMetrics)}
            style={{
              width: 28,
              height: 14,
              borderRadius: 7,
              border: 'none',
              cursor: 'pointer',
              background: shareMetrics ? 'var(--accent-amber)' : 'var(--bg-hover)',
              position: 'relative',
            }}
          >
            <span style={{
              position: 'absolute',
              top: 1,
              left: shareMetrics ? 14 : 1,
              width: 12,
              height: 12,
              borderRadius: '50%',
              background: 'var(--bg-base)',
              transition: 'left 0.15s',
            }} />
          </button>
          <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>Share metrics with Ring</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <button
            onClick={() => setAllowProactive(!allowProactive)}
            style={{
              width: 28,
              height: 14,
              borderRadius: 7,
              border: 'none',
              cursor: 'pointer',
              background: allowProactive ? 'var(--accent-amber)' : 'var(--bg-hover)',
              position: 'relative',
            }}
          >
            <span style={{
              position: 'absolute',
              top: 1,
              left: allowProactive ? 14 : 1,
              width: 12,
              height: 12,
              borderRadius: '50%',
              background: 'var(--bg-base)',
              transition: 'left 0.15s',
            }} />
          </button>
          <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>Allow proactive messages</span>
        </div>
        <button onClick={savePrivacy} style={{ ...smallBtn, marginTop: 8 }}>SAVE PRIVACY</button>
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
            onClick={handleExport}
            style={{ ...smallBtn, background: 'var(--bg-hover)', color: 'var(--text-primary)', border: '1px solid var(--border)' }}
          >
            EXPORT
          </button>
          <button
            onClick={handleReset}
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
      <ConfirmModal
        open={confirmDialog !== null}
        title={confirmDialog?.title ?? ''}
        message={confirmDialog?.message ?? ''}
        variant={confirmDialog?.variant}
        on_confirm={() => { confirmDialog?.action(); setConfirmDialog(null) }}
        on_cancel={() => setConfirmDialog(null)}
      />
    </div>
  )
}
