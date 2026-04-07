import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../../api/client'
import { RingList } from './RingList'
import { CreateRing } from './CreateRing'
import type { RingListItem, CreateRingRequest } from '../../types'

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

  if (loading) return <p>Loading...</p>
  if (error) return <p role="alert">{error}</p>

  return (
    <div>
      <h1>Ring Group</h1>
      <CreateRing on_create={handle_create} />
      <RingList rings={rings} on_select={handle_select} />
    </div>
  )
}
