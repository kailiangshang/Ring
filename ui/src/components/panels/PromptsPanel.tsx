import { useState, useEffect, useRef } from 'react'
import { api } from '../../services/api'

interface PromptEntry {
  module: string
  name: string
  content: string
}

interface TreeNode {
  label: string
  key: string
  children?: TreeNode[]
  entry?: PromptEntry
}

const LAYER_MAP: Record<string, { layer: string; label: string }> = {
  super_ring: { layer: 'Super Ring', label: 'Super Ring' },
  search: { layer: 'Super Ring', label: 'Super Ring' },
  group_ring: { layer: 'Group Ring', label: 'Group Ring' },
  archive: { layer: 'Group Ring', label: 'Group Ring' },
  blueprint: { layer: 'Group Ring', label: 'Group Ring' },
  workflow: { layer: 'Group Ring', label: 'Group Ring' },
  group_docs: { layer: 'Group Ring', label: 'Group Ring' },
  export: { layer: 'Group Ring', label: 'Group Ring' },
  compact: { layer: 'Group Ring', label: 'Group Ring' },
  session_skill: { layer: 'Session Ring', label: 'Session Ring' },
  self_chat: { layer: 'Self', label: 'Self' },
}

function buildTree(prompts: PromptEntry[]): TreeNode[] {
  const groups = new Map<string, PromptEntry[]>()
  for (const p of prompts) {
    const layer = LAYER_MAP[p.module]?.layer ?? 'Other'
    const list = groups.get(layer) ?? []
    list.push(p)
    groups.set(layer, list)
  }

  const order = ['Super Ring', 'Group Ring', 'Session Ring', 'Self', 'Other']
  const moduleLabels: Record<string, string> = {
    super_ring: '核心提示词',
    search: '跨 Ring 检索',
    group_ring: '核心提示词',
    archive: '归档判断',
    blueprint: '蓝图构建',
    workflow: '工作流工具',
    group_docs: '群组文档维护',
    export: '导出报告',
    compact: '上下文压缩',
    session_skill: 'Skill 提示词',
    self_chat: '核心提示词',
  }

  return order
    .filter((l) => groups.has(l))
    .map((layer) => {
      const entries = groups.get(layer)!
      const subGroups = new Map<string, PromptEntry[]>()
      for (const e of entries) {
        const list = subGroups.get(e.module) ?? []
        list.push(e)
        subGroups.set(e.module, list)
      }

      const children: TreeNode[] = []
      for (const [mod, modEntries] of subGroups) {
        if (modEntries.length === 1 && modEntries[0].name === 'system') {
          children.push({
            label: moduleLabels[mod] ?? mod,
            key: `${mod}.${modEntries[0].name}`,
            entry: modEntries[0],
          })
        } else {
          children.push({
            label: moduleLabels[mod] ?? mod,
            key: mod,
            children: modEntries.map((e) => ({
              label: e.name.replace(/_/g, ' '),
              key: `${mod}.${e.name}`,
              entry: e,
            })),
          })
        }
      }

      return { label: layer, key: layer, children }
    })
}

function TreeItem({
  node,
  depth,
  selected,
  onSelect,
  expanded,
  onToggle,
}: {
  node: TreeNode
  depth: number
  selected: string | null
  onSelect: (key: string) => void
  expanded: Set<string>
  onToggle: (key: string) => void
}) {
  const isLeaf = !!node.entry
  const isSelected = selected === node.key
  const isExpanded = expanded.has(node.key)
  const hasChildren = node.children && node.children.length > 0

  return (
    <>
      <div
        onClick={() => {
          if (isLeaf) {
            onSelect(node.key)
          } else if (hasChildren) {
            onToggle(node.key)
          }
        }}
        style={{
          paddingLeft: depth * 14 + 8,
          paddingRight: 8,
          paddingTop: 5,
          paddingBottom: 5,
          cursor: isLeaf || hasChildren ? 'pointer' : 'default',
          background: isSelected
            ? 'var(--bg-hover)'
            : 'transparent',
          borderLeft: isSelected
            ? '2px solid var(--accent-cyan)'
            : '2px solid transparent',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}
      >
        {hasChildren && (
          <span
            style={{
              fontSize: 9,
              color: 'var(--text-dim)',
              transition: 'transform 0.15s',
              transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)',
              display: 'inline-block',
              width: 10,
              textAlign: 'center',
            }}
          >
            ▶
          </span>
        )}
        {!hasChildren && <span style={{ width: 10 }} />}
        <span
          style={{
            fontSize: isLeaf ? 11 : 12,
            fontWeight: isLeaf ? 400 : 700,
            color: isLeaf
              ? isSelected
                ? 'var(--accent-cyan)'
                : 'var(--text-secondary)'
              : 'var(--text-primary)',
          }}
        >
          {node.label}
        </span>
      </div>
      {hasChildren &&
        isExpanded &&
        node.children!.map((child) => (
          <TreeItem
            key={child.key}
            node={child}
            depth={depth + 1}
            selected={selected}
            onSelect={onSelect}
            expanded={expanded}
            onToggle={onToggle}
          />
        ))}
    </>
  )
}

export function PromptsModal({ onClose }: { onClose: () => void }) {
  const [prompts, setPrompts] = useState<PromptEntry[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [expanded, setExpanded] = useState<Set<string>>(new Set(['Super Ring', 'Group Ring', 'Session Ring', 'Self']))
  const backdropRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    api.get<PromptEntry[]>('/prompts').then(setPrompts).catch(() => {})
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', handleKey)
    return () => window.removeEventListener('keydown', handleKey)
  }, [onClose])

  const tree = buildTree(prompts)
  const active = prompts.find((p) => `${p.module}.${p.name}` === selected)

  const toggleExpand = (key: string) => {
    setExpanded((prev) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }

  return (
    <div
      ref={backdropRef}
      onClick={(e) => {
        if (e.target === backdropRef.current) onClose()
      }}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0, 0, 0, 0.6)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
    >
      <div
        style={{
          width: '50vw',
          height: '50vh',
          minWidth: 560,
          minHeight: 380,
          background: 'var(--bg-panel)',
          border: '1px solid var(--border)',
          borderRadius: 6,
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          boxShadow: '0 8px 32px rgba(0,0,0,0.4)',
        }}
      >
        <div
          style={{
            height: 36,
            minHeight: 36,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '0 12px',
            borderBottom: '1px solid var(--border)',
          }}
        >
          <span style={{ fontSize: 13, fontWeight: 700, color: 'var(--accent-ice)' }}>
            AI Prompts
          </span>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--text-dim)',
              fontSize: 16,
              cursor: 'pointer',
              padding: '0 4px',
            }}
          >
            ✕
          </button>
        </div>
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          <div
            style={{
              width: 220,
              minWidth: 220,
              borderRight: '1px solid var(--border)',
              overflow: 'auto',
            }}
          >
            {tree.map((node) => (
              <TreeItem
                key={node.key}
                node={node}
                depth={0}
                selected={selected}
                onSelect={setSelected}
                expanded={expanded}
                onToggle={toggleExpand}
              />
            ))}
          </div>
          <div style={{ flex: 1, overflow: 'auto', padding: 14 }}>
            {active ? (
              <>
                <div style={{ marginBottom: 10, display: 'flex', gap: 8, alignItems: 'center' }}>
                  <span style={{ fontSize: 10, fontWeight: 700, color: 'var(--accent-cyan)', letterSpacing: '0.05em' }}>
                    {LAYER_MAP[active.module]?.label ?? active.module}
                  </span>
                  <span style={{ color: 'var(--text-dim)', fontSize: 10 }}>·</span>
                  <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>{active.module}</span>
                  <span style={{ color: 'var(--text-dim)', fontSize: 10 }}>·</span>
                  <span style={{ fontSize: 10, color: 'var(--text-secondary)' }}>{active.name}</span>
                </div>
                <pre
                  style={{
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                    fontFamily: 'Cascadia Code, monospace',
                    fontSize: 11,
                    lineHeight: 1.7,
                    color: 'var(--text-primary)',
                    margin: 0,
                  }}
                >
                  {active.content}
                </pre>
              </>
            ) : (
              <div style={{ color: 'var(--text-dim)', textAlign: 'center', paddingTop: 60, fontSize: 12 }}>
                Select a prompt to view its content
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
