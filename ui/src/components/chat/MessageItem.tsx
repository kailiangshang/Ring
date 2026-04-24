import { useState, useRef, useEffect } from 'react'
import type { ChatMessage } from '../../types/chat'
import { useChatStore } from '../../stores/chat-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useGraphStore } from '../../stores/graph-store'

const COLLAPSE_HEIGHT = 200

interface ExtractedConcept {
  label: string
  node_type: string
  tags: string[]
}

interface ExtractionData {
  summary?: string
  concepts: ExtractedConcept[]
  relations: { from: string; to: string; relation: string }[]
  suggested_graph?: string
}

function parseExtraction(text: string, tag: string): ExtractionData | null {
  const re = new RegExp(`<${tag}>\\s*([\\s\\S]*?)\\s*<\\/${tag}>`)
  const match = text.match(re)
  if (!match) return null
  try {
    return JSON.parse(match[1])
  } catch {
    return null
  }
}

function stripExtractionTags(text: string): string {
  return text
    .replace(/<file_analysis>[\s\S]*?<\/file_analysis>/g, '')
    .replace(/<knowledge_extraction>[\s\S]*?<\/knowledge_extraction>/g, '')
    .trim()
}

function ExtractionCard({
  data,
  onAddToGraph,
}: {
  data: ExtractionData
  onAddToGraph: () => void
}) {
  return (
    <div
      style={{
        background: 'var(--bg-input)',
        border: '1px solid var(--border)',
        borderRadius: 6,
        padding: 10,
        marginTop: 8,
      }}
    >
      {data.summary && (
        <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginBottom: 6 }}>
          {data.summary}
        </div>
      )}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4, marginBottom: 6 }}>
        {data.concepts.map((c, i) => (
          <span
            key={i}
            style={{
              fontSize: 9,
              background:
                c.node_type === 'category'
                  ? 'rgba(34,211,238,0.15)'
                  : c.node_type === 'leaf'
                    ? 'rgba(52,211,153,0.15)'
                    : 'rgba(167,139,250,0.15)',
              padding: '2px 6px',
              borderRadius: 3,
              color: 'var(--text-secondary)',
            }}
          >
            {c.label}
          </span>
        ))}
      </div>
      {data.relations.length > 0 && (
        <div style={{ fontSize: 9, color: 'var(--text-dim)', lineHeight: 1.6, marginBottom: 6 }}>
          {data.relations.slice(0, 5).map((r, i) => (
            <div key={i}>
              {r.from} <span style={{ color: 'var(--accent-cyan)' }}>&rarr;</span> {r.to}{' '}
              <span style={{ color: 'var(--text-dim)' }}>({r.relation})</span>
            </div>
          ))}
        </div>
      )}
      <button
        onClick={onAddToGraph}
        style={{
          fontSize: 9,
          fontWeight: 700,
          background: 'var(--accent-cyan)',
          color: 'var(--bg-base)',
          border: 'none',
          borderRadius: 3,
          padding: '3px 8px',
          cursor: 'pointer',
        }}
      >
        添加到图谱
      </button>
    </div>
  )
}

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  group_ring: 'var(--accent-cyan)',
  super_ring: 'var(--accent-cyan)',
  session_ring: 'var(--accent-teal)',
  self: 'var(--accent-amber)',
  system: 'var(--accent-green)',
}

interface MessageItemProps {
  message: ChatMessage
}

export function MessageItem({ message }: MessageItemProps) {
  const streaming_message_id = useChatStore((s) => s.streaming_message_id)
  const isStreaming = message.id === streaming_message_id
  const labelColor = ROLE_COLORS[message.role] ?? 'var(--text-muted)'
  const label = message.role === 'user' ? 'YOU' : message.sender_name.toUpperCase()
  const isUser = message.role === 'user'

  const contentRef = useRef<HTMLDivElement>(null)
  const [collapsed, setCollapsed] = useState(false)
  const [overflowing, setOverflowing] = useState(false)

  useEffect(() => {
    const el = contentRef.current
    if (!el) return
    if (el.scrollHeight > COLLAPSE_HEIGHT + 40) {
      setOverflowing(true)
      setCollapsed(true)
    }
  }, [message.content])

  useEffect(() => {
    if (isStreaming) setCollapsed(false)
  }, [isStreaming])

  const isAi = !isUser && message.role !== 'system'

  const activeRingId = useRingStore((s) => s.active_ring_id)
  const fileAnalysis = isAi ? parseExtraction(message.content, 'file_analysis') : null
  const knowledgeExtraction = isAi ? parseExtraction(message.content, 'knowledge_extraction') : null
  const displayContent = (fileAnalysis || knowledgeExtraction) ? stripExtractionTags(message.content) : message.content
  const hasExtraction = !!(fileAnalysis || knowledgeExtraction)

  const isFileCard = message.role === 'system' && message.content.startsWith('📎 ')
  const fileCardMatch = isFileCard ? message.content.match(/^📎 (.+)\n---\n([\s\S]*)$/) : null
  const fileCardFilename = fileCardMatch ? fileCardMatch[1] : ''
  const fileCardContent = fileCardMatch ? fileCardMatch[2] : ''

  const rings = useRingStore((s) => s.rings)
  const selectRing = useRingStore((s) => s.selectRing)
  const setContext = useAppStore((s) => s.setContext)

  const handleCitationClick = (ringName: string) => {
    const ring = rings.find((r) => r.name === ringName)
    if (!ring) return
    selectRing(ring.id)
    setContext('ring')
  }

  /* eslint-disable @typescript-eslint/no-explicit-any */
  const mdComponents: Record<string, any> = {
    p(props: any) {
      const text = Array.isArray(props.children)
        ? props.children.join('')
        : String(props.children ?? '')
      const citationRegex = /\[([^\]]+ > [^\]]+)\]/g
      const parts: Array<{ text: string; citation?: { ringName: string; title: string; match: string } }> = []
      let lastIndex = 0
      let match: RegExpExecArray | null

      while ((match = citationRegex.exec(text)) !== null) {
        if (match.index > lastIndex) {
          parts.push({ text: text.slice(lastIndex, match.index) })
        }
        const [full, ref] = match
        const sep = ref.indexOf(' > ')
        const ringName = ref.slice(0, sep).trim()
        const title = ref.slice(sep + 3).trim()
        parts.push({ text: '', citation: { ringName, title, match: full } })
        lastIndex = match.index + full.length
      }
      if (lastIndex < text.length) {
        parts.push({ text: text.slice(lastIndex) })
      }

      if (parts.length === 0) {
        return <p style={{ margin: '0 0 8px' }}>{props.children}</p>
      }

      return (
        <p style={{ margin: '0 0 8px' }}>
          {parts.map((part, i) =>
            part.citation ? (
              <a
                key={i}
                href="#"
                onClick={(e) => {
                  e.preventDefault()
                  handleCitationClick(part.citation!.ringName)
                }}
                style={{
                  color: 'var(--accent-teal)',
                  textDecoration: 'none',
                  cursor: 'pointer',
                  fontWeight: 600,
                  borderBottom: '1px dashed var(--accent-teal)',
                }}
                title={`Go to Ring: ${part.citation.ringName}`}
              >
                {part.citation.match}
              </a>
            ) : (
              <span key={i}>{part.text}</span>
            )
          )}
        </p>
      )
    },
    h1(props: any) { return <h1 style={{ fontSize: 18, fontWeight: 700, color: 'var(--accent-ice)', margin: '12px 0 6px' }}>{props.children}</h1> },
    h2(props: any) { return <h2 style={{ fontSize: 15, fontWeight: 700, color: 'var(--accent-ice)', margin: '10px 0 4px' }}>{props.children}</h2> },
    h3(props: any) { return <h3 style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-ice)', margin: '8px 0 4px' }}>{props.children}</h3> },
    pre(props: any) { return <div style={{ margin: '6px 0' }}>{props.children}</div> },
    ul(props: any) { return <ul style={{ margin: '4px 0', paddingLeft: 20 }}>{props.children}</ul> },
    ol(props: any) { return <ol style={{ margin: '4px 0', paddingLeft: 20 }}>{props.children}</ol> },
    li(props: any) { return <li style={{ marginBottom: 2 }}>{props.children}</li> },
    blockquote(props: any) { return <blockquote style={{ borderLeft: '3px solid var(--accent-cyan)', margin: '6px 0', paddingLeft: 10, color: 'var(--text-secondary)' }}>{props.children}</blockquote> },
    strong(props: any) { return <strong style={{ fontWeight: 700, color: 'var(--text-primary)' }}>{props.children}</strong> },
    hr() { return <hr style={{ border: 'none', borderTop: '1px solid var(--border)', margin: '8px 0' }} /> },
    code(props: any) {
      const { className, children } = props
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
    a(props: any) {
      return <a href={props.href} style={{ color: 'var(--accent-teal)', textDecoration: 'underline' }} target="_blank" rel="noreferrer">{props.children}</a>
    },
    table(props: any) { return <table style={{ borderCollapse: 'collapse', margin: '6px 0', fontSize: 11, width: '100%' }}>{props.children}</table> },
    th(props: any) { return <th style={{ border: '1px solid var(--border)', padding: '4px 8px', textAlign: 'left', fontWeight: 700, color: 'var(--accent-ice)' }}>{props.children}</th> },
    td(props: any) { return <td style={{ border: '1px solid var(--border)', padding: '4px 8px' }}>{props.children}</td> },
  }
  /* eslint-enable @typescript-eslint/no-explicit-any */

  return (
    <div style={{
      padding: '8px 16px',
      borderBottom: '1px solid var(--border)',
      display: 'flex',
      justifyContent: isUser ? 'flex-end' : 'flex-start',
    }}>
      <div style={{
        maxWidth: '85%',
        background: isUser ? 'var(--bg-active)' : 'transparent',
        borderRadius: isUser ? '6px 6px 2px 6px' : 0,
        padding: isUser ? '8px 12px' : 0,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, justifyContent: isUser ? 'flex-end' : 'flex-start' }}>
          <span style={{ fontSize: 10, fontWeight: 700, color: labelColor, letterSpacing: '0.1em' }}>
            {label}
          </span>
          <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
            {new Date(message.created_at).toLocaleTimeString()}
          </span>
        </div>
        {isFileCard && fileCardMatch && (
          <div style={{
            border: '1px solid var(--border)',
            borderRadius: 6,
            padding: '8px 12px',
            background: 'var(--bg-active)',
            marginBottom: 8,
            fontSize: 13,
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <span style={{ fontSize: 14 }}>📎</span>
              <span style={{ fontWeight: 700, color: 'var(--accent-ice)', fontSize: 12 }}>
                {fileCardFilename}
              </span>
            </div>
            <div
              ref={contentRef}
              style={{
                color: 'var(--text-secondary)',
                fontSize: 12,
                lineHeight: 1.5,
                maxHeight: collapsed ? 200 : undefined,
                overflow: collapsed ? 'hidden' : 'visible',
                position: 'relative',
              }}
            >
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word', fontFamily: 'inherit' }}>
                {fileCardContent.length > 500 ? fileCardContent.slice(0, 500) + '...' : fileCardContent}
              </pre>
              {collapsed && (
                <div
                  style={{
                    position: 'absolute',
                    bottom: 0,
                    left: 0,
                    right: 0,
                    height: 40,
                    background: 'linear-gradient(transparent, var(--bg-active))',
                    display: 'flex',
                    alignItems: 'flex-end',
                    justifyContent: 'center',
                    cursor: 'pointer',
                  }}
                  onClick={() => setCollapsed(false)}
                >
                  <span style={{ fontSize: 10, color: 'var(--accent-cyan)', fontWeight: 700, paddingBottom: 4 }}>
                    EXPAND
                  </span>
                </div>
              )}
            </div>
          </div>
        )}
        {!isFileCard && (
        <div
          ref={contentRef}
          style={{
            color: 'var(--text-primary)',
            lineHeight: 1.6,
            fontSize: 13,
            maxHeight: collapsed ? COLLAPSE_HEIGHT : undefined,
            overflow: collapsed ? 'hidden' : 'visible',
            position: 'relative',
            transition: 'max-height 0.2s ease',
          }}
        >
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={mdComponents}>
            {displayContent}
          </ReactMarkdown>
          {isStreaming && (
            <span style={{
              display: 'inline-block',
              width: 6,
              height: 14,
              background: 'var(--accent-cyan)',
              marginLeft: 2,
              verticalAlign: 'middle',
              animation: 'blink 1s step-end infinite',
            }} />
          )}
          {collapsed && overflowing && (
            <div
              style={{
                position: 'absolute',
                bottom: 0,
                left: 0,
                right: 0,
                height: 40,
                background: 'linear-gradient(transparent, var(--bg-base))',
                display: 'flex',
                alignItems: 'flex-end',
                justifyContent: 'center',
                cursor: 'pointer',
              }}
              onClick={() => setCollapsed(false)}
            >
              <span style={{
                fontSize: 10,
                color: 'var(--accent-cyan)',
                fontWeight: 700,
                paddingBottom: 4,
                letterSpacing: '0.05em',
              }}>
                EXPAND
              </span>
            </div>
          )}
        </div>
        )}
        {hasExtraction && (
          <>
            {fileAnalysis && (
              <ExtractionCard
                data={fileAnalysis}
                onAddToGraph={() => {
                  if (activeRingId) {
                    useGraphStore.getState().createNodesFromExtraction(
                      activeRingId,
                      fileAnalysis.concepts,
                      fileAnalysis.relations,
                    )
                  }
                }}
              />
            )}
            {knowledgeExtraction && (
              <ExtractionCard
                data={knowledgeExtraction}
                onAddToGraph={() => {
                  if (activeRingId) {
                    useGraphStore.getState().createNodesFromExtraction(
                      activeRingId,
                      knowledgeExtraction.concepts,
                      knowledgeExtraction.relations,
                    )
                  }
                }}
              />
            )}
          </>
        )}
        {!collapsed && overflowing && isAi && (
          <button
            onClick={() => setCollapsed(true)}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--accent-cyan)',
              fontSize: 10,
              fontWeight: 700,
              cursor: 'pointer',
              padding: '2px 0',
              letterSpacing: '0.05em',
            }}
          >
            COLLAPSE
          </button>
        )}
        {message.token_usage && message.role !== 'user' && (
          <div style={{ marginTop: 4, fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8 }}>
            {message.token_usage.prompt_tokens !== undefined && (
              <span>prompt: {message.token_usage.prompt_tokens}</span>
            )}
            {message.token_usage.completion_tokens !== undefined && (
              <span>completion: {message.token_usage.completion_tokens}</span>
            )}
            {message.token_usage.total_tokens !== undefined && (
              <span>total: {message.token_usage.total_tokens}</span>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
