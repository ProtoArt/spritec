use std::sync::Arc;
use std::collections::VecDeque;

use crate::math::Mat4;

use super::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub fn from_gltf(node: &gltf::Node) -> Self {
        NodeId(node.index())
    }
}

#[derive(Debug, Clone)]
struct NodeTreeEntry {
    /// The data for this node
    node: Arc<Node>,
    /// The children of this node
    ///
    /// Each child's global transform is dependent on the parent node's transform
    children: Arc<Vec<NodeId>>,
}

#[derive(Debug)]
pub struct NodeTree {
    /// The nodes and their children, indexed by the node ID
    nodes: Vec<NodeTreeEntry>,
}

impl NodeTree {
    /// Creates a node tree from the nodes and their children, ordered by node ID
    pub fn from_ordered_nodes<N>(nodes: N) -> Self
        where N: Iterator<Item=(Node, Vec<NodeId>)>
    {
        Self {
            nodes: nodes.map(|(node, children)| NodeTreeEntry {
                node: Arc::new(node),
                children: Arc::new(children),
            }).collect(),
        }
    }

    /// Returns a new copy of this tree with certain nodes replaced. Note that this does not
    /// change the structure of the node hierarchy. It only replaces certain nodes with other
    /// nodes.
    ///
    /// The `replace` function takes a node and returns either a new version of it or `None`.
    /// If `None` is returned, no replacement is performed.
    ///
    /// Note: Even if no replacements are actually performed, this function *always* makes a
    /// complete copy of the tree. The copy is relatively cheap since the nodes themselves are
    /// reference counted, but it still requires allocating memory to store the new copies of
    /// the nodes. Try to avoid calling this method if you know in advance that no replacements
    /// need to be made.
    pub fn with_replacements<R>(&self, mut replace: R) -> Self
        where R: FnMut(&Node) -> Option<Node>
    {
        let Self {nodes} = self;

        Self {
            nodes: nodes.iter().map(|entry| match replace(&entry.node) {
                Some(node) => NodeTreeEntry {
                    node: Arc::new(node),
                    children: entry.children.clone(),
                },
                // This clone should be cheap because we use Arc in NodeTreeEntry
                None => entry.clone(),
            }).collect(),
        }
    }

    /// Get the node with the given ID
    pub fn get(&self, node_id: NodeId) -> &Node {
        &self.entry(node_id).node
    }

    /// Iterate over the child nodes of the given node
    pub fn children(&self, node_id: NodeId) -> impl Iterator<Item=&Node> {
        self.entry(node_id).children.iter().map(move |&id| self.get(id))
    }

    /// Traverse a node hierarchy, treating the given node as a root node, yielding each node and
    /// the world transform of that node's parent. Note that since this is a world transform, it
    /// will reflect the total transformation up the entire hierarchy.
    pub fn traverse(&self, node: NodeId) -> TraverseNodes {
        let mut queue = VecDeque::new();
        queue.push_back((Mat4::identity(), node));
        TraverseNodes {nodes: self, queue}
    }

    fn entry(&self, node_id: NodeId) -> &NodeTreeEntry {
        let NodeId(idx) = node_id;
        &self.nodes[idx]
    }
}

pub struct TraverseNodes<'a> {
    nodes: &'a NodeTree,
    /// A queue of each node to be traversed, and its parent transform
    queue: VecDeque<(Mat4, NodeId)>,
}

impl<'a> Iterator for TraverseNodes<'a> {
    type Item = (Mat4, &'a Node);

    fn next(&mut self) -> Option<Self::Item> {
        // This code assumes that the node hierarchy is not cyclic
        let (parent_trans, node_id) = self.queue.pop_front()?;
        let node = self.nodes.get(node_id);

        let world_transform = parent_trans * node.transform;
        self.queue.extend(self.nodes.children(node_id)
            .map(|node| (world_transform, node.id)));

        Some((parent_trans, node))
    }
}
