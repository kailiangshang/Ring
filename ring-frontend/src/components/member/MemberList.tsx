import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useMemberStore } from '../../stores/memberStore'
import { Button } from '../../components/ui/Button'
import { Badge } from '../../components/ui/Badge'
import { Input } from '../../components/ui/Input'
import { Avatar } from '../../components/ui/Avatar'
import { EmptyState } from '../../components/ui/EmptyState'
import './MemberList.css'

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

  return (
    <div className="member-list">
      <div className="member-list-header">
        <h2 className="member-list-title">Members</h2>
        <Button onClick={handle_generate_invite}>Invite</Button>
      </div>

      {invite_token && (
        <div className="member-invite-card">
          Invite link: {window.location.origin}/join?token={invite_token}
        </div>
      )}

      {error && <p className="setup-error" role="alert">{error}</p>}
      {loading && <p>Loading...</p>}

      {!loading && members.length === 0 && (
        <EmptyState
          icon="👥"
          title="No members yet"
          description="Invite members to your Ring to start collaborating."
        />
      )}

      {!loading && members.map((m) => (
        <div key={m.id} className="member-card">
          <Avatar name={m.display_name} size="md" />
          <div className="member-card-info">
            <div className="member-card-name">{m.display_name}</div>
            <div className="member-card-joined">Joined {m.joined_at}</div>
          </div>
          <Badge status={m.role}>{m.role}</Badge>
          {m.role !== 'creator' && (
            <div className="member-card-actions">
              <Input
                input_type="select"
                onChange={(e) => {
                  if (ringId && e.target.value)
                    update_role(ringId, m.id, e.target.value)
                }}
                defaultValue=""
              >
                <option value="" disabled>Change role</option>
                <option value="admin">Admin</option>
                <option value="member">Member</option>
                <option value="readonly">Readonly</option>
              </Input>
              <Button
                size="sm"
                variant="danger"
                onClick={() => {
                  if (ringId && confirm(`Remove ${m.display_name}?`))
                    remove_member(ringId, m.id)
                }}
              >
                Remove
              </Button>
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
