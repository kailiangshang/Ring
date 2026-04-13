import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { useGraphStore } from '../../stores/graphStore'
import { ForceGraph } from '../../components/graph/ForceGraph'
import { Input } from '../../components/ui/Input'
import { Button } from '../../components/ui/Button'
import './GraphView.css'

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
    <div className="graph-view">
      <div className="graph-toolbar">
        <Input
          input_type="select"
          value={current_graph_id || ''}
          onChange={(e) => {
            if (ringId && e.target.value) select_graph(ringId, e.target.value)
          }}
        >
          {graphs.map((g) => (
            <option key={g} value={g}>{g}</option>
          ))}
        </Input>
        <Button size="sm" onClick={() => set_show_add_form(!show_add_form)}>+ Node</Button>
      </div>

      {show_add_form && (
        <div className="graph-add-bar">
          <Input
            value={new_label}
            onChange={(e) => set_new_label(e.target.value)}
            placeholder="Node label"
            className="graph-add-input"
          />
          <Input
            input_type="select"
            value={new_type}
            onChange={(e) => set_new_type(e.target.value)}
          >
            <option value="concept">Concept</option>
            <option value="category">Category</option>
            <option value="document">Document</option>
            <option value="event">Event</option>
            <option value="person">Person</option>
            <option value="task">Task</option>
          </Input>
          <Button size="sm" onClick={handle_add_node}>Create</Button>
        </div>
      )}

      <div className="graph-body">
        {loading && <p className="graph-loading">Loading graph...</p>}
        {error && <p className="graph-error" role="alert">{error}</p>}
        <ForceGraph
          nodes={nodes}
          edges={edges}
          on_node_click={(node_id) => {
            if (ringId && current_graph_id) select_node(ringId, current_graph_id, node_id)
          }}
          selected_node_id={selected_node_id}
        />

        {selected_node_content && (
          <div className="graph-detail-panel">
            <h3>{selected_node_content.label}</h3>
            {selected_node_content.content && (
              <pre className="graph-detail-content">{selected_node_content.content}</pre>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
