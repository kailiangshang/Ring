DROP TABLE IF EXISTS search_index;

CREATE VIRTUAL TABLE search_index USING fts5(
    source_type,
    source_id,
    ring_id,
    ring_name,
    title,
    content,
    metadata,
    tokenize='unicode61'
);

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'message', m.id, m.ring_id, COALESCE(r.name, ''), m.sender_name, m.content,
    json_object('role', m.role)
FROM messages m
LEFT JOIN rings r ON m.ring_id = r.id
WHERE m.ring_id IS NOT NULL AND m.ring_id != 'super';

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'session_message', sm.id, s.ring_id, r.name, sm.sender_name, sm.content,
    json_object('session_id', sm.session_id, 'message_type', sm.message_type)
FROM session_messages sm
JOIN sessions s ON sm.session_id = s.id
JOIN rings r ON s.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'graph_node', gn.id, gn.ring_id, r.name, gn.label,
    gn.content || ' ' || gn.tags,
    json_object('node_type', gn.node_type, 'graph_id', gn.graph_id)
FROM graph_nodes gn
JOIN rings r ON gn.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'session', s.id, s.ring_id, r.name, s.title,
    COALESCE(s.description, '') || ' ' || COALESCE(s.summary, ''),
    json_object('skill', s.skill, 'phase', s.phase)
FROM sessions s
JOIN rings r ON s.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'group_doc', gd.ring_id || ':' || gd.doc_name, gd.ring_id, r.name, gd.doc_name, gd.content,
    '{}'
FROM group_docs gd
JOIN rings r ON gd.ring_id = r.id;
