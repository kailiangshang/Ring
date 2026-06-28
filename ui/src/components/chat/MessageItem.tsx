import React, { useState, useRef, useEffect } from 'react'
import type { ChatMessage } from '../../types/chat'
import { useChatStore } from '../../stores/chat-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import { usePanelStore } from '../../stores/panel-store'
import { MarkdownRenderer } from '../common/MarkdownRenderer'
import { useGraphStore } from '../../stores/graph-store'
import { ConfirmModal } from '../common/ConfirmModal'

const COLLAPSE_HEIGHT = 200

interface ExtractedConcept {
  label: string
  node_type: string
  tags: string[]
  match?: string | null
}

interface ExtractionData {
  summary?: string
  concepts: ExtractedConcept[]
  relations: { from: string; to: string; relation: string }[]
  suggested_graph?: string
}

interface GraphActionData {
  intent: string
}

const handledGraphActionMessages = new Set<string>()

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
    .replace(/<graph_action>[\s\S]*?<\/graph_action>/g, '')
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
              display: 'inline-flex',
              alignItems: 'center',
              gap: 3,
              background: c.match
                ? 'rgba(52,211,153,0.15)'
                : c.node_type === 'category'
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
            {c.match ? (
              <span style={{
                fontSize: 8,
                background: 'rgba(52,211,153,0.3)',
                padding: '0 3px',
                borderRadius: 2,
                color: 'var(--accent-green, #34d399)',
              }}>
                关联已有节点
              </span>
            ) : null}
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

function MessageItemInner({ message }: MessageItemProps) {
  const streaming_message_id = useChatStore((s) => s.streaming_message_id)
  const selection_mode = useChatStore((s) => s.selection_mode)
  const selected_messages = useChatStore((s) => s.selected_messages)
  const toggleMessageSelection = useChatStore((s) => s.toggleMessageSelection)
  const enterSelectionMode = useChatStore((s) => s.enterSelectionMode)
  const isSelected = selected_messages.includes(message.id)
  const isStreaming = message.id === streaming_message_id
  const labelColor = ROLE_COLORS[message.role] ?? 'var(--text-muted)'
  const label = message.role === 'user' ? 'YOU' : message.sender_name.toUpperCase()
  const isUser = message.role === 'user'

  const contentRef = useRef<HTMLDivElement>(null)
  const [collapsed, setCollapsed] = useState(false)
  const effectiveCollapsed = isStreaming ? false : collapsed
  const [overflowing, setOverflowing] = useState(false)

  useEffect(() => {
    if (isStreaming) {
      return
    }
    const el = contentRef.current
    if (!el) return
    requestAnimationFrame(() => {
      if (el.scrollHeight > COLLAPSE_HEIGHT + 40) {
        setOverflowing(true)
      }
    })
  }, [message.content, isStreaming])

  const isAi = !isUser && message.role !== 'system'

  const activeRingId = useRingStore((s) => s.active_ring_id)
  const fileAnalysis = isAi ? parseExtraction(message.content, 'file_analysis') : null
  const knowledgeExtraction = isAi ? parseExtraction(message.content, 'knowledge_extraction') : null
  const hasExtraction = !!(fileAnalysis || knowledgeExtraction)

  // Collapse file content in user messages: show filename only, hide full text
  const FILE_PATTERN = /📎 File: (.+)\n---\n[\s\S]*?(?=\n\n📎 File:|$)/g
  const hasFileContent = FILE_PATTERN.test(message.content)
  let displayContent = message.content
  if (hasFileContent && !isAi) {
    // Replace each file block with just the filename tag
    displayContent = message.content.replace(FILE_PATTERN, '📎 $1')
  }
  if (fileAnalysis || knowledgeExtraction) {
    displayContent = stripExtractionTags(displayContent)
  }

  const isFileCard = message.role === 'system' && message.content.startsWith('📎 ')
  const fileCardMatch = isFileCard ? message.content.match(/^📎 (.+)\n---\n([\s\S]*)$/) : null
  const fileCardFilename = fileCardMatch ? fileCardMatch[1] : ''
  const fileCardContent = fileCardMatch ? fileCardMatch[2] : ''

  const rings = useRingStore((s) => s.rings)
  const selectRing = useRingStore((s) => s.selectRing)
  const setContext = useAppStore((s) => s.setContext)
  const graphId = useGraphStore((s) => s.graph_id)
  const graphs = useGraphStore((s) => s.graphs)
  const addMessage = useChatStore((s) => s.addMessage)
  const graphAction = isAi ? parseExtraction(message.content, 'graph_action') as GraphActionData | null : null
  const activeGraphName = graphs.find((g) => g.id === graphId)?.name ?? 'main'
  const [graphConfirmClosed, setGraphConfirmClosed] = useState(() => handledGraphActionMessages.has(message.id))
  const graphConfirmOpen =
    Boolean(knowledgeExtraction && graphAction?.intent === 'confirm_create_graph') &&
    !graphConfirmClosed &&
    !handledGraphActionMessages.has(message.id)

  const closeGraphConfirm = () => {
    handledGraphActionMessages.add(message.id)
    setGraphConfirmClosed(true)
  }

  const handleCitationClick = (ringName: string) => {
    const ring = rings.find((r) => r.name === ringName)
    if (!ring) return
    selectRing(ring.id)
    setContext('ring')
  }

  /* eslint-disable @typescript-eslint/no-explicit-any */
  const citationP: any = (props: any) => {
      const text = Array.isArray(props.children)
        ? props.children.map((c: any) => (typeof c === 'string' ? c : '')).join('')
        : String(props.children ?? '')
      const citationRegex = /\[([^\]]+ > [^\]]+)\]/g

      // Fast path: no citation pattern → render as-is to avoid [object Object]
      if (!citationRegex.test(text)) {
        return <p style={{ margin: '0 0 8px' }}>{props.children}</p>
      }

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
    }
  /* eslint-enable @typescript-eslint/no-explicit-any */

  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault()
    if (!selection_mode) {
      enterSelectionMode(message.id)
    }
  }

  const handleTouchStart = () => {
    longPressTimer.current = setTimeout(() => {
      if (!selection_mode) {
        enterSelectionMode(message.id)
      }
    }, 500)
  }

  const handleTouchEnd = () => {
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current)
      longPressTimer.current = null
    }
  }

  const handleClick = () => {
    if (selection_mode) {
      toggleMessageSelection(message.id)
    }
  }

  return (
    <div
      onContextMenu={handleContextMenu}
      onTouchStart={handleTouchStart}
      onTouchEnd={handleTouchEnd}
      onClick={handleClick}
      style={{
      padding: '8px 16px',
      borderBottom: '1px solid var(--border)',
      display: 'flex',
      justifyContent: isUser ? 'flex-end' : 'flex-start',
      borderLeft: isSelected ? '3px solid var(--accent-cyan)' : undefined,
      background: isSelected ? 'rgba(34, 211, 238, 0.05)' : undefined,
      cursor: selection_mode ? 'pointer' : undefined,
      position: 'relative',
    }}>
      {selection_mode && (
        <div style={{
          position: 'absolute',
          left: 4,
          top: '50%',
          transform: 'translateY(-50%)',
          width: 18,
          height: 18,
          borderRadius: '50%',
          border: isSelected ? 'none' : '2px solid var(--text-dim)',
          background: isSelected ? 'var(--accent-cyan)' : 'transparent',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 10,
          color: isSelected ? 'var(--bg-base)' : 'transparent',
          fontWeight: 700,
          zIndex: 1,
        }}>
          ✓
        </div>
      )}
      <div style={{
        maxWidth: '85%',
        background: isUser ? 'var(--bg-active)' : 'var(--bg-input)',
        borderRadius: isUser ? '6px 6px 2px 6px' : '2px 6px 6px 6px',
        padding: isUser ? '8px 12px' : '8px 12px',
        border: isUser ? 'none' : '1px solid var(--border)',
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
                maxHeight: effectiveCollapsed ? 200 : undefined,
                overflow: effectiveCollapsed ? 'hidden' : 'visible',
                position: 'relative',
              }}
            >
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word', fontFamily: 'inherit' }}>
                {fileCardContent}
              </pre>
              {effectiveCollapsed && (
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
            {!effectiveCollapsed && fileCardContent.length > 500 && (
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
                }}
              >
                COLLAPSE
              </button>
            )}
          </div>
        )}
        {!isFileCard && (
        <div
          ref={contentRef}
          style={{
            color: 'var(--text-primary)',
            lineHeight: 1.6,
            fontSize: 13,
            maxHeight: effectiveCollapsed ? COLLAPSE_HEIGHT : undefined,
            overflow: effectiveCollapsed ? 'hidden' : 'visible',
            position: 'relative',
            transition: 'max-height 0.2s ease',
          }}
        >
          <MarkdownRenderer content={displayContent} components={{ p: citationP }} />
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
          {effectiveCollapsed && overflowing && (
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
                fontSize: 11,
                color: 'var(--accent-cyan)',
                fontWeight: 700,
                padding: '4px 12px',
                letterSpacing: '0.05em',
                background: 'var(--bg-base)',
                borderRadius: 4,
                border: '1px solid var(--accent-cyan)',
                cursor: 'pointer',
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
                      graphId,
                    )
                  }
                }}
              />
            )}
          </>
        )}
        {!effectiveCollapsed && overflowing && isAi && (
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
      <ConfirmModal
        open={graphConfirmOpen}
        title="Create Graph Nodes"
        message={
          knowledgeExtraction
            ? `Create ${knowledgeExtraction.concepts.length} nodes and ${knowledgeExtraction.relations.length} relations in graph "${activeGraphName}"?`
            : ''
        }
        confirm_label="Create"
        cancel_label="Cancel"
        on_cancel={closeGraphConfirm}
        on_confirm={() => {
          closeGraphConfirm()
          if (!activeRingId || !knowledgeExtraction) return
          useGraphStore
            .getState()
            .createNodesFromExtraction(
              activeRingId,
              knowledgeExtraction.concepts,
              knowledgeExtraction.relations,
              graphId,
            )
            .then(({ createdNodes, createdEdges }) => {
              usePanelStore.getState().open('graph')
              addMessage({
                id: `sys-${crypto.randomUUID()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: `Graph updated: ${createdNodes} node(s), ${createdEdges} edge(s) created in ${activeGraphName}.`,
                created_at: new Date().toISOString(),
              })
            })
            .catch((error: unknown) => {
              const msg = error instanceof Error ? error.message : 'failed to create graph nodes'
              addMessage({
                id: `sys-${crypto.randomUUID()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: `Graph creation failed: ${msg}`,
                created_at: new Date().toISOString(),
              })
            })
        }}
      />
    </div>
  )
}

export const MessageItem = React.memo(MessageItemInner)
