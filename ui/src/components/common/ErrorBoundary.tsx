import { Component, type ReactNode } from 'react'

interface Props {
  children: ReactNode
  fallback?: ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }
      return (
        <div
          style={{
            padding: 24,
            color: 'var(--accent-red, #f87171)',
            textAlign: 'center',
            fontSize: 14,
          }}
        >
          <h3 style={{ marginBottom: 8 }}>Something went wrong</h3>
          <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
            {this.state.error?.message || 'Unknown error'}
          </p>
          <button
            onClick={() => window.location.reload()}
            style={{
              marginTop: 16,
              padding: '6px 16px',
              background: 'var(--accent-cyan)',
              border: 'none',
              borderRadius: 4,
              color: '#000',
              cursor: 'pointer',
              fontSize: 12,
            }}
          >
            Reload
          </button>
        </div>
      )
    }

    return this.props.children
  }
}
