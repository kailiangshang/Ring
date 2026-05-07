import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { Components } from 'react-markdown'

/* eslint-disable @typescript-eslint/no-explicit-any */
const defaultComponents: Components = {
  p({ children }) { return <p style={{ margin: '0 0 8px' }}>{children}</p> },
  h1({ children }) { return <h1 style={{ fontSize: 18, fontWeight: 700, color: 'var(--accent-ice)', margin: '12px 0 6px' }}>{children}</h1> },
  h2({ children }) { return <h2 style={{ fontSize: 15, fontWeight: 700, color: 'var(--accent-ice)', margin: '10px 0 4px' }}>{children}</h2> },
  h3({ children }) { return <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-ice)', margin: '8px 0 4px' }}>{children}</h3> },
  pre({ children }) { return <div style={{ margin: '6px 0' }}>{children}</div> },
  ul({ children }) { return <ul style={{ margin: '4px 0', paddingLeft: 20 }}>{children}</ul> },
  ol({ children }) { return <ol style={{ margin: '4px 0', paddingLeft: 20 }}>{children}</ol> },
  li({ children }) { return <li style={{ marginBottom: 2 }}>{children}</li> },
  blockquote({ children }) { return <blockquote style={{ borderLeft: '3px solid var(--accent-cyan)', margin: '6px 0', paddingLeft: 10, color: 'var(--text-secondary)' }}>{children}</blockquote> },
  strong({ children }) { return <strong style={{ fontWeight: 700, color: 'var(--text-primary)' }}>{children}</strong> },
  hr() { return <hr style={{ border: 'none', borderTop: '1px solid var(--border)', margin: '8px 0' }} /> },
  code({ className, children }: any) {
    if (className?.startsWith('language-')) {
      return (
        <pre style={{
          background: 'var(--bg-base)',
          border: '1px solid var(--border)',
          borderRadius: 3,
          padding: '8px 12px',
          fontSize: 12,
          overflow: 'auto',
          margin: '6px 0',
        }}>
          <code>{children}</code>
        </pre>
      )
    }
    return (
      <code style={{
        background: 'var(--bg-base)',
        padding: '1px 4px',
        borderRadius: 3,
        fontSize: 12,
        color: 'var(--accent-teal)',
      }}>{children}</code>
    )
  },
  a({ href, children }) {
    return <a href={href} style={{ color: 'var(--accent-teal)', textDecoration: 'underline' }} target="_blank" rel="noreferrer">{children}</a>
  },
  table({ children }) { return <table style={{ borderCollapse: 'collapse', margin: '6px 0', fontSize: 11, width: '100%' }}>{children}</table> },
  th({ children }) { return <th style={{ border: '1px solid var(--border)', padding: '4px 8px', textAlign: 'left', fontWeight: 700, color: 'var(--accent-ice)' }}>{children}</th> },
  td({ children }) { return <td style={{ border: '1px solid var(--border)', padding: '4px 8px' }}>{children}</td> },
}
/* eslint-enable @typescript-eslint/no-explicit-any */

interface MarkdownRendererProps {
  content: string
  components?: Partial<Components>
}

export function MarkdownRenderer({ content, components }: MarkdownRendererProps) {
  const merged = components ? { ...defaultComponents, ...components } : defaultComponents
  return (
    <ReactMarkdown remarkPlugins={[remarkGfm]} components={merged}>
      {content}
    </ReactMarkdown>
  )
}

export { defaultComponents }
