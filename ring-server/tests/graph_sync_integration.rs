use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::types::{GraphJson, NewEdge, NewNode};

fn new_store() -> PetgraphStore {
    PetgraphStore::new()
}

#[tokio::test]
async fn create_node_updates_graph_json() {
    let store = new_store();
    let node = store
        .create_node(
            "graph-1",
            NewNode {
                label: "NodeA".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: Some("desc".into()),
            },
        )
        .await
        .unwrap();

    let exported = store.export_graph_json("graph-1").await.unwrap();
    assert_eq!(exported.nodes.len(), 1);
    assert_eq!(exported.edges.len(), 0);

    let exported_node = &exported.nodes[0];
    assert_eq!(exported_node.id, node.id);
    assert_eq!(exported_node.label, "NodeA");
    assert_eq!(exported_node.node_type, "concept");
    assert_eq!(exported_node.description, Some("desc".into()));
    assert_eq!(exported_node.graph_id, "graph-1");
}

#[tokio::test]
async fn delete_node_removes_from_graph_json() {
    let store = new_store();
    let node = store
        .create_node(
            "graph-1",
            NewNode {
                label: "ToRemove".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();

    store.delete_node("graph-1", &node.id).await.unwrap();

    let exported = store.export_graph_json("graph-1").await.unwrap();
    assert!(exported.nodes.is_empty());
    assert!(exported.edges.is_empty());
}

#[tokio::test]
async fn import_export_round_trip() {
    let store1 = new_store();
    let n1 = store1
        .create_node(
            "graph-1",
            NewNode {
                label: "Alpha".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();
    let n2 = store1
        .create_node(
            "graph-1",
            NewNode {
                label: "Beta".into(),
                node_type: "topic".into(),
                parent_id: None,
                description: Some("second node".into()),
            },
        )
        .await
        .unwrap();
    store1
        .create_edge(
            "graph-1",
            NewEdge {
                source_id: n1.id.clone(),
                target_id: n2.id.clone(),
                relation: "related_to".into(),
                label: Some("link".into()),
            },
        )
        .await
        .unwrap();

    let exported1 = store1.export_graph_json("graph-1").await.unwrap();

    let store2 = new_store();
    store2
        .import_graph_json("graph-1", &exported1)
        .await
        .unwrap();
    let exported2 = store2.export_graph_json("graph-1").await.unwrap();

    assert_eq!(exported2.nodes.len(), 2);
    assert_eq!(exported2.edges.len(), 1);

    let mut nodes1: Vec<_> = exported1.nodes.into_iter().map(|n| n.id).collect();
    let mut nodes2: Vec<_> = exported2.nodes.into_iter().map(|n| n.id).collect();
    nodes1.sort();
    nodes2.sort();
    assert_eq!(nodes1, nodes2);

    let edge2 = &exported2.edges[0];
    assert_eq!(edge2.source_id, n1.id);
    assert_eq!(edge2.target_id, n2.id);
    assert_eq!(edge2.relation, "related_to");
    assert_eq!(edge2.label, Some("link".into()));
}

#[tokio::test]
async fn multiple_graphs_independent() {
    let store = new_store();

    let ga = store
        .create_node(
            "graph-a",
            NewNode {
                label: "NodeA".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();
    let gb = store
        .create_node(
            "graph-b",
            NewNode {
                label: "NodeB".into(),
                node_type: "topic".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();

    let export_a = store.export_graph_json("graph-a").await.unwrap();
    let export_b = store.export_graph_json("graph-b").await.unwrap();

    assert_eq!(export_a.nodes.len(), 1);
    assert_eq!(export_a.nodes[0].id, ga.id);
    assert_eq!(export_b.nodes.len(), 1);
    assert_eq!(export_b.nodes[0].id, gb.id);

    let a_ids: Vec<&str> = export_a.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(!a_ids.contains(&gb.id.as_str()));

    let b_ids: Vec<&str> = export_b.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(!b_ids.contains(&ga.id.as_str()));
}

#[tokio::test]
async fn edge_survives_round_trip() {
    let store1 = new_store();
    let n1 = store1
        .create_node(
            "graph-1",
            NewNode {
                label: "Src".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();
    let n2 = store1
        .create_node(
            "graph-1",
            NewNode {
                label: "Tgt".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();
    let edge = store1
        .create_edge(
            "graph-1",
            NewEdge {
                source_id: n1.id.clone(),
                target_id: n2.id.clone(),
                relation: "depends_on".into(),
                label: Some("critical".into()),
            },
        )
        .await
        .unwrap();

    let exported = store1.export_graph_json("graph-1").await.unwrap();

    let store2 = new_store();
    store2
        .import_graph_json("graph-1", &exported)
        .await
        .unwrap();

    let imported = store2.export_graph_json("graph-1").await.unwrap();
    assert_eq!(imported.edges.len(), 1);

    let imported_edge = &imported.edges[0];
    assert_eq!(imported_edge.source_id, n1.id);
    assert_eq!(imported_edge.target_id, n2.id);
    assert_eq!(imported_edge.relation, "depends_on");
    assert_eq!(imported_edge.label, Some("critical".into()));
    assert_eq!(imported_edge.graph_id, "graph-1");
    assert_eq!(imported_edge.id, edge.id);
}

#[tokio::test]
async fn import_replaces_existing_data() {
    let store = new_store();

    let old_node = store
        .create_node(
            "graph-1",
            NewNode {
                label: "OldNode".into(),
                node_type: "concept".into(),
                parent_id: None,
                description: None,
            },
        )
        .await
        .unwrap();

    let new_data = GraphJson {
        nodes: vec![ring_server::graph::types::NodeData {
            id: "fresh-id".into(),
            label: "FreshNode".into(),
            node_type: "topic".into(),
            parent_id: None,
            description: Some("replaced".into()),
            graph_id: "graph-1".into(),
            markdown_path: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        }],
        edges: vec![],
    };

    store.import_graph_json("graph-1", &new_data).await.unwrap();

    let exported = store.export_graph_json("graph-1").await.unwrap();
    assert_eq!(exported.nodes.len(), 1);

    let exported_ids: Vec<&str> = exported.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(!exported_ids.contains(&old_node.id.as_str()));

    assert_eq!(exported.nodes[0].id, "fresh-id");
    assert_eq!(exported.nodes[0].label, "FreshNode");

    let old_fetch = store.get_node("graph-1", &old_node.id).await.unwrap();
    assert!(old_fetch.is_none());
}
