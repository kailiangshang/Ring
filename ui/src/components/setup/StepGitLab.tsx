import type { SetupData } from './SetupWizard'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
  error: string | null
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  fontSize: 13,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 12,
  marginTop: 4,
}

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function StepGitLab({ data, onChange, onNext, onBack, error }: StepProps) {
  return (
    <div style={{ padding: '20px', maxWidth: 420, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 4 }}>
        Step 3: GitLab Config
        <span style={{ fontSize: 12, color: 'var(--text-dim)', fontWeight: 400, marginLeft: 8 }}>(Optional)</span>
      </h2>

      <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.6, marginBottom: 16 }}>
        GitLab 用于归档对话记录、创建合并请求和团队协作。配置后你可以：
        <ul style={{ margin: '4px 0', paddingLeft: 16 }}>
          <li>将 AI 对话自动归档为 Markdown 文件</li>
          <li>通过 Git 管理知识图谱的变更</li>
          <li>团队成员提交归档审核</li>
        </ul>
        不配置 GitLab 仍可正常使用 Ring，但归档功能将不可用。
      </div>

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input
        value={data.gitlab_url}
        onChange={(e) => onChange({ gitlab_url: e.target.value })}
        placeholder="https://gitlab.company.com"
        style={inputStyle}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Personal Access Token
        <span style={{ color: 'var(--text-muted)', fontWeight: 400 }}>
          {' '}— 在 GitLab Settings → Access Tokens 中创建，需勾选 <strong>api</strong> 权限
        </span>
      </label>
      <input
        type="password"
        value={data.gitlab_token}
        onChange={(e) => onChange({ gitlab_token: e.target.value })}
        placeholder="glpat-xxx"
        style={inputStyle}
      />

      {error && (
        <div style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 8 }}>
          {error}
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          style={{
            ...navButtonStyle,
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            marginLeft: 'auto',
          }}
        >
          Done
        </button>
        <button
          onClick={onNext}
          style={{
            ...navButtonStyle,
            opacity: 0.7,
          }}
        >
          Skip
        </button>
      </div>
    </div>
  )
}
