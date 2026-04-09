import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../../api/client'
import { Skeleton } from '../../components/ui/Skeleton'
import { RingList } from './RingList'
import { CreateRing } from './CreateRing'
import type { RingListItem, CreateRingRequest } from '../../types'
import './RingHub.css'

export function RingHub() {
  const [rings, set_rings] = useState<RingListItem[]>([])
  const [loading, set_loading] = useState(true)
  const [error, set_error] = useState<string | null>(null)
  const navigate = useNavigate()

  useEffect(() => {
    load_rings()
  }, [])

  const load_rings = async () => {
    set_loading(true)
    try {
      const data = await api.list_rings()
      set_rings(data)
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_loading(false)
    }
  }

  const handle_create = async (req: CreateRingRequest) => {
    await api.create_ring(req)
    await load_rings()
  }

  const handle_select = (id: string) => {
    navigate(`/ring/${id}`)
  }

  return (
    <div className="ring-hub">
      <div className="ring-hub-header">
        <div>
          <h1 className="ring-hub-title">Ring Hub</h1>
          <p className="ring-hub-subtitle">你的群组知识协作空间</p>
        </div>
        <CreateRing on_create={handle_create} />
      </div>

      {error && <p className="setup-error" role="alert">{error}</p>}

      {loading ? (
        <div className="ring-hub-grid">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} width="100%" height="120px" />
          ))}
        </div>
      ) : (
        <RingList rings={rings} on_select={handle_select} />
      )}

      <div className="ring-hub-footer">对话记录仅保存在当前设备</div>
    </div>
  )
}
