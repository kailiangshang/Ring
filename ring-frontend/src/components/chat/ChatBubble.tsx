import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import { useEffect, useRef, useMemo } from 'react'
import mermaid from 'mermaid'
import './ChatView.css'

mermaid.initialize({ startOnLoad: false, theme: 'default' })

interface ChatBubbleProps {
  role: 'user' | 'assistant'
  content: string
}

function MermaidBlock({ code }: { code: string }) {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!containerRef.current) return
    const el = containerRef.current
    const id = `m-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`
    let mounted = true
    mermaid
      .render(id, code)
      .then(({ svg }) => {
        if (mounted) el.innerHTML = svg
      })
      .catch((err) => {
        if (mounted) {
          console.error('mermaid render error:', err)
          el.style.whiteSpace = 'pre-wrap'
          el.style.color = 'var(--color-text-primary)'
          el.style.fontSize = '0.85em'
          el.textContent = code
        }
      })
    return () => {
      mounted = false
      const svgEl = document.getElementById(id)
      if (svgEl) svgEl.remove()
    }
  }, [code])

  return <div ref={containerRef} className="chat-bubble-mermaid" />
}

type Segment = { type: 'text'; content: string } | { type: 'mermaid'; content: string }

function splitMermaid(content: string): Segment[] {
  const segments: Segment[] = []
  const regex = /```mermaid\n([\s\S]*?)```/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = regex.exec(content)) !== null) {
    if (match.index > lastIndex) {
      segments.push({ type: 'text', content: content.slice(lastIndex, match.index) })
    }
    segments.push({ type: 'mermaid', content: match[1].trim() })
    lastIndex = regex.lastIndex
  }

  if (lastIndex < content.length) {
    segments.push({ type: 'text', content: content.slice(lastIndex) })
  }

  return segments
}

export function ChatBubble({ role, content }: ChatBubbleProps) {
  const is_user = role === 'user'

  const segments = useMemo(() => {
    if (is_user) return []
    return splitMermaid(content)
  }, [content, is_user])

  return (
    <div className={`chat-bubble-row ${is_user ? 'chat-bubble-row-user' : 'chat-bubble-row-assistant'}`}>
      <div className={`chat-bubble ${is_user ? 'chat-bubble-user' : 'chat-bubble-assistant'}`}>
        {is_user ? (
          <span>{content}</span>
        ) : segments.length > 0 ? (
          segments.map((seg, i) =>
            seg.type === 'mermaid' ? (
              <MermaidBlock key={i} code={seg.content} />
            ) : (
              <ReactMarkdown
                key={i}
                remarkPlugins={[remarkGfm]}
                rehypePlugins={[rehypeHighlight]}
              >
                {seg.content}
              </ReactMarkdown>
            ),
          )
        ) : (
          <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
            {content}
          </ReactMarkdown>
        )}
      </div>
    </div>
  )
}
