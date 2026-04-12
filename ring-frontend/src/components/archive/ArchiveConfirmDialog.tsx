import { useState } from 'react'
import { Button } from '../ui/Button'
import { Modal } from '../ui/Modal'
import { NodeTree } from '../graph/NodeTree'
import type { GraphNode } from '../../types'
import './ArchiveConfirmDialog.css'

interface ArchiveConfirmDialogProps {
  open: boolean
  on_close: () => void
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  nodes: GraphNode[]
  on_confirm: (target_node_id: string | undefined) => Promise<void>
  loading?: boolean
}

export function ArchiveConfirmDialog({
  open,
  on_close,
  suggested_title,
  suggested_parent,
  nodes,
  on_confirm,
  loading,
}: ArchiveConfirmDialogProps) {
  const [show_selector, set_show_selector] = useState(false)
  const [selected_parent_id, set_selected_parent_id] = useState<string | undefined>(
    suggested_parent?.id,
  )
  const [error, set_error] = useState<string | null>(null)
  const [confirming, set_confirming] = useState(false)

  const handle_confirm = async () => {
    set_error(null)
    set_confirming(true)
    try {
      await on_confirm(selected_parent_id)
      on_close()
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_confirming(false)
    }
  }

  const selected_node = nodes.find((n) => n.id === selected_parent_id)

  return (
    <Modal
      open={open}
      on_close={on_close}
      title="确认归档"
      footer={
        <>
          <Button variant="secondary" onClick={on_close}>取消</Button>
          <Button onClick={handle_confirm} disabled={loading || confirming}>确认归档</Button>
        </>
      }
    >
      {error && <p className="archive-confirm-error" role="alert">{error}</p>}

      {suggested_title && (
        <div>
          <div className="archive-confirm-label">标题</div>
          <div className="archive-confirm-value">{suggested_title}</div>
        </div>
      )}

      <div className="archive-confirm-placement">
        <div className="archive-confirm-placement-header">
          <span className="archive-confirm-placement-title">节点位置</span>
          <button
            className="archive-confirm-change-btn"
            onClick={() => set_show_selector(!show_selector)}
          >
            {show_selector ? '收起' : '更改位置'}
          </button>
        </div>
        <div className="archive-confirm-selected-node">
          {selected_node ? selected_node.label : '(根节点下新建)'}
        </div>
        {show_selector && (
          <div className="archive-confirm-node-selector">
            <div
              style={{ padding: '4px 8px', cursor: 'pointer', fontSize: 'var(--font-size-sm)', color: selected_parent_id === undefined ? 'var(--color-accent)' : 'inherit' }}
              onClick={() => set_selected_parent_id(undefined)}
            >
              (根节点下新建)
            </div>
            <NodeTree
              nodes={nodes}
              selected_node_id={selected_parent_id ?? null}
              on_select={(id) => {
                set_selected_parent_id(id)
                set_show_selector(false)
              }}
            />
          </div>
        )}
      </div>
    </Modal>
  )
}
