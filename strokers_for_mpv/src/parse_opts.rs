use mpv_client::Node;
use std::collections::HashMap;

pub(crate) fn get_node_string_map(node: Node) -> HashMap<String, String> {
    match node {
        Node::Map(node) => node
            .into_iter()
            .filter_map(|(k, v)| match v {
                Node::String(s) => Some((k, s)),
                Node::Int(i) => Some((k, i.to_string())),
                Node::Double(f) => Some((k, f.to_string())),
                Node::Bool(b) => Some((k, b.to_string())),
                _ => None,
            })
            .collect(),
        _ => HashMap::new(),
    }
}
