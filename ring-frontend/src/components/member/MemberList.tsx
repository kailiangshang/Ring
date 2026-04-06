import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMemberStore } from '../../stores/memberStore'

export function MemberList() {
  const { ringId } = useParams<{ ringId: string }>()
  const { members, loading, error, load_members, update_role, remove_member } =
    useMemberStore()
  const [invite_token, set_invite_token] = useState<string | null>(null)

  useEffect(() => {
    if (ringId) load_members(ringId)
  }, [ringId, load_members])

  const handle_generate_invite = async () => {
    if (!ringId) return
    const token = await useMemberStore
      .getState()
      .generate_invite(ringId, {
        token_type: 'open',
        role: 'member',
        max_uses: 10,
      })
    set_invite_token(token)
  }

  const role_badge_color = (role: string) => {
    switch (role) {
      case 'creator':
        return '#e6b800'
      case 'admin':
        return '#0366d6'
      case 'member':
        return '#28a745'
      case 'readonly':
        return '#888'
      default:
        return '#888'
    }
  }

  return (
    <div style={{ padding: '1.5rem', maxWidth: '800px', margin: '0 auto' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
        <h2>Members</h2>
        <button
          onClick={handle_generate_invite}
          style={{
            padding: '0.5rem 1rem',
            background: '#0366d6',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Generate Invite
        </button>
      </div>

      {invite_token && (
        <div
          style={{
            padding: '0.75rem',
            background: '#d4edda',
            borderRadius: '4px',
            marginBottom: '1rem',
            fontFamily: 'monospace',
            fontSize: '0.85rem',
            wordBreak: 'break-all',
          }}
        >
          Invite link: {window.location.origin}/join?token={invite_token}
        </div>
      )}

      {error && <p style={{ color: 'red' }}>{error}</p>}
      {loading && <p>Loading...</p>}

      {!loading && members.length === 0 && (
        <p style={{ color: '#888' }}>No members yet</p>
      )}

      {!loading && members.length > 0 && (
        <table
          style={{
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: '0.9rem',
          }}
        >
          <thead>
            <tr style={{ borderBottom: '2px solid #ddd', textAlign: 'left' }}>
              <th style={{ padding: '0.5rem' }}>#</th>
              <th style={{ padding: '0.5rem' }}>Name</th>
              <th style={{ padding: '0.5rem' }}>Role</th>
              <th style={{ padding: '0.5rem' }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {members.map((m) => (
              <tr
                key={m.id}
                style={{ borderBottom: '1px solid #eee' }}
              >
                <td style={{ padding: '0.5rem' }}>#{m.token_id}</td>
                <td style={{ padding: '0.5rem' }}>{m.display_name}</td>
                <td style={{ padding: '0.5rem' }}>
                  <span
                    style={{
                      padding: '2px 8px',
                      borderRadius: '3px',
                      fontSize: '0.75rem',
                      fontWeight: 600,
                      color: '#fff',
                      background: role_badge_color(m.role),
                    }}
                  >
                    {m.role}
                  </span>
                </td>
                <td style={{ padding: '0.5rem' }}>
                  {m.role !== 'creator' && (
                    <div style={{ display: 'flex', gap: '0.5rem' }}>
                      <select
                        onChange={(e) => {
                          if (ringId && e.target.value)
                            update_role(ringId, m.id, e.target.value)
                        }}
                        defaultValue=""
                        style={{ fontSize: '0.8rem', padding: '2px 4px' }}
                      >
                        <option value="" disabled>
                          Change role
                        </option>
                        <option value="admin">Admin</option>
                        <option value="member">Member</option>
                        <option value="readonly">Readonly</option>
                      </select>
                      <button
                        onClick={() => {
                          if (ringId && confirm(`Remove ${m.display_name}?`))
                            remove_member(ringId, m.id)
                        }}
                        style={{
                          padding: '2px 8px',
                          background: '#dc3545',
                          color: '#fff',
                          border: 'none',
                          borderRadius: '3px',
                          cursor: 'pointer',
                          fontSize: '0.8rem',
                        }}
                      >
                        Remove
                      </button>
                    </div>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}
