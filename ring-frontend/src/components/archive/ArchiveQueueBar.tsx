import { useEffect } from 'react'
import { useGitStore } from '../../stores/gitStore'
import './ArchiveQueueBar.css'

interface ArchiveQueueBarProps {
  ring_id: string
}

export function ArchiveQueueBar({ ring_id }: ArchiveQueueBarProps) {
  const { archive_queue, load_archive_queue } = useGitStore()

  useEffect(() => {
    load_archive_queue(ring_id)
    const interval = setInterval(() => load_archive_queue(ring_id), 30000)
    return () => clearInterval(interval)
  }, [ring_id, load_archive_queue])

  if (!archive_queue) return null

  const { current_review, queue } = archive_queue
  const has_activity = current_review || queue.length > 0

  if (!has_activity) {
    return (
      <div className="archive-queue-bar">
        <span className="archive-queue-bar-empty">归档队列空闲</span>
      </div>
    )
  }

  return (
    <div className="archive-queue-bar">
      {current_review && (
        <div className="archive-queue-bar-item">
          <span className="archive-queue-bar-dot" />
          <span>正在审核: {current_review.title} (by {current_review.author})</span>
        </div>
      )}
      {queue.length > 0 && (
        <div className="archive-queue-bar-item">
          <span>排队中: {queue.length} 个</span>
        </div>
      )}
    </div>
  )
}
