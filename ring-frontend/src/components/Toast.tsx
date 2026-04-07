import { useEffect, useState, useCallback, useRef } from 'react'

interface ToastItem {
  id: number
  message: string
}

let add_toast_fn: ((message: string) => void) | null = null

export function toast_error(message: string) {
  if (add_toast_fn) add_toast_fn(message)
}

export function Toast() {
  const [toasts, set_toasts] = useState<ToastItem[]>([])
  const counter = useRef(0)

  const add = useCallback((message: string) => {
    const id = ++counter.current
    set_toasts((prev) => [...prev, { id, message }])
    setTimeout(() => {
      set_toasts((prev) => prev.filter((t) => t.id !== id))
    }, 4000)
  }, [])

  useEffect(() => {
    add_toast_fn = add
    return () => {
      add_toast_fn = null
    }
  }, [add])

  if (toasts.length === 0) return null

  return (
    <div className="toast-container">
      {toasts.map((t) => (
        <div key={t.id} className="toast-item">
          {t.message}
        </div>
      ))}
    </div>
  )
}
