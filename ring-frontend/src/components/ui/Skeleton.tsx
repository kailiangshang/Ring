import './Skeleton.css'

interface SkeletonProps { width?: string; height?: string }

export function Skeleton({ width = '100%', height = '16px' }: SkeletonProps) {
  return <div className="skeleton" style={{ width, height }} />
}
