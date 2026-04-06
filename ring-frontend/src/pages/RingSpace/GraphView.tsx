import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useGraphStore } from '../../stores/graphStore'
import { ForceGraph } from '../../components/graph/ForceGraph'
import { NodeTree } from '../../components/graph/NodeTree'

export function GraphView() {
  const { ringId } = useParams<{ ringId: string }>()
  const {
    graphs,
    current_graph_id,
    nodes,
    edges,
    selected_node_id,
    selected_node_content,
    loading,
    error,
    load_graphs,
    select_graph,
    select_node,
    create_node,
    reset,
  } = useGraphStore()

  const [show_add_form, set_show_add_form] = useState(false)
  const [new_label, set_new_label] = useState('')
  const [new_type, set_new_type] = useState('concept')

  useEffect(() => {
    if (!ringId) return
    const init = async () => {
      reset()
      await load_graphs(ringId)
    }
    init()
  }, [ringId])

  useEffect(() => {
    if (!ringId || graphs.length === 0 || current_graph_id) return
    select_graph(ringId, graphs[0])
  }, [graphs, ringId, current_graph_id])

  const handle_add_node = async () => {
    if (!ringId || !current_graph_id || !new_label.trim()) return
    await create_node(ringId, current_graph_id, {
      label: new_label.trim(),
      node_type: new_type,
    })
    set_new_label('')
    set_show_add_form(false)
  }

  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <div
        style={{
          width: 240,
          borderRight: '1px solid #e5e7eb',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        <div style={{ padding: 8 }}>
          <select
            value={current_graph_id || ''}
            onChange={(e) => {
              if (ringId && e.target.value) select_graph(ringId, e.target.value)
            }}
            style={{ width: '100%', padding: 4 }}
          >
            {graphs.map((g) => (
              <option key={g} value={g}>
                {g}
              </option>
            ))}
          </select>
        </div>
        <div style={{ flex: 1, overflow: 'auto' }}>
          <NodeTree
            nodes={nodes}
            selected_node_id={selected_node_id}
            on_select={(node_id) => {
              if (ringId && current_graph_id) select_node(ringId, current_graph_id, node_id)
            }}
          />
        </div>
        <div style={{ padding: 8, borderTop: '1px solid #e5e7eb' }}>
          <button onClick={() => set_show_add_form(!show_add_form)}>
            Add Node
          </button>
          {show_add_form && (
            <div style={{ marginTop: 8 }}>
              <input
                value={new_label}
                onChange={(e) => set_new_label(e.target.value)}
                placeholder="Node label"
                style={{ width: '100%', marginBottom: 4, padding: 4 }}
              />
              <select
                value={new_type}
                onChange={(e) => set_new_type(e.target.value)}
                style={{ width: '100%', marginBottom: 4, padding: 4 }}
              >
                <option value="concept">Concept</option>
                <option value="category">Category</option>
                <option value="document">Document</option>
                <option value="event">Event</option>
                <option value="person">Person</option>
                <option value="task">Task</option>
              </select>
              <button onClick={handle_add_node}>Create</button>
            </div>
          )}
        </div>
      </div>

      <div style={{ flex: 1, position: 'relative' }}>
        {loading && <p>Loading graph...</p>}
        {error && <p role="alert">{error}</p>}
        <ForceGraph
          nodes={nodes}
          edges={edges}
          on_node_click={(node_id) => {
            if (ringId && current_graph_id) select_node(ringId, current_graph_id, node_id)
          }}
          selected_node_id={selected_node_id}
        />
      </div>

      {selected_node_content && (
        <div
          style={{
            width: 280,
            borderLeft: '1px solid #e5e7eb',
            padding: 16,
            overflow: 'auto',
          }}
        >
          <h3>{selected_node_content.label}</h3>
          {selected_node_content.content && (
            <pre style={{ whiteSpace: 'pre-wrap', fontSize: 13 }}>
              {selected_node_content.content}
            </pre>
          )}
        </div>
      )}
    </div>
  )
}
