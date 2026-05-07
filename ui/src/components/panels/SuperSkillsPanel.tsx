import { useEffect, useState } from 'react'
import type { SkillInfo } from '../../types/skill'
import { listSkills, installSkill, removeSkill } from '../../services/api'
import { ConfirmModal } from '../common/ConfirmModal'

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '6px 10px',
  color: 'var(--text-primary)',
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 8,
  marginTop: 2,
}

export function SuperSkillsPanel() {
  const [skills, setSkills] = useState<SkillInfo[]>([])
  const [installName, setInstallName] = useState('')
  const [installUrl, setInstallUrl] = useState('')
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null)
  const [confirmDialog, setConfirmDialog] = useState<{ title: string; message: string; action: () => void; variant?: 'danger' | 'default' } | null>(null)

  const load = () => {
    listSkills().then((res) => setSkills(res.skills)).catch(() => {})
  }

  useEffect(() => { load() }, [])

  const handleInstall = async () => {
    if (!installName.trim() || !installUrl.trim()) return
    setMsg(null)
    try {
      const result = await installSkill(installName.trim(), installUrl.trim())
      setMsg({ ok: result.ok, text: result.ok ? `Installed: ${result.name}` : 'Install failed' })
      setInstallName('')
      setInstallUrl('')
      load()
    } catch (e: unknown) {
      setMsg({ ok: false, text: e instanceof Error ? e.message : 'Failed' })
    }
  }

  const handleRemove = (name: string) => {
    setConfirmDialog({
      title: 'Remove Skill',
      message: `Remove skill "${name}"?`,
      variant: 'danger',
      action: async () => {
        try {
          await removeSkill(name)
          load()
        } catch (e) {
          setMsg({ ok: false, text: e instanceof Error ? e.message : 'Failed to remove' })
        }
      },
    })
  }

  return (
    <div style={{ fontSize: 12 }}>
      {skills.length === 0 && (
        <p style={{ color: 'var(--text-dim)', marginBottom: 12 }}>No skills installed</p>
      )}
      {skills.map((s) => (
        <div key={s.name} style={{ padding: '6px 0', borderBottom: '1px solid var(--border)' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--accent-ice)', fontWeight: 600 }}>{s.name}</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{ fontSize: 9, color: s.source === 'builtin' ? 'var(--accent-cyan)' : 'var(--accent-amber)' }}>
                {s.source === 'builtin' ? 'built-in' : 'user'}
              </span>
              {s.source !== 'builtin' && (
                <span style={{ fontSize: 9, color: 'var(--text-dim)', cursor: 'pointer' }} onClick={() => handleRemove(s.name)}>
                  remove
                </span>
              )}
            </div>
          </div>
          <div style={{ color: 'var(--text-muted)', fontSize: 11, marginTop: 2 }}>{s.description}</div>
        </div>
      ))}

      <div style={{ marginTop: 16, paddingTop: 12, borderTop: '1px solid var(--border)' }}>
        <p style={{ color: 'var(--text-secondary)', fontWeight: 700, marginBottom: 8, fontSize: 11 }}>Install New Skill</p>
        <label style={{ fontSize: 10, color: 'var(--text-dim)' }}>Name</label>
        <input value={installName} onChange={(e) => setInstallName(e.target.value)} style={inputStyle} />
        <label style={{ fontSize: 10, color: 'var(--text-dim)' }}>Source URL</label>
        <input value={installUrl} onChange={(e) => setInstallUrl(e.target.value)} placeholder="https://..." style={inputStyle} />
        <button
          onClick={handleInstall}
          disabled={!installName.trim() || !installUrl.trim()}
          style={{
            width: '100%',
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 3,
            padding: '6px 0',
            fontSize: 10,
            fontWeight: 700,
            cursor: 'pointer',
            opacity: (!installName.trim() || !installUrl.trim()) ? 0.4 : 1,
          }}
        >
          INSTALL
        </button>
        {msg && (
          <div style={{
            marginTop: 8,
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
