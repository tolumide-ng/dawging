// use std::collections::HashMap;
use std::{collections::HashMap, rc::Rc, cell::RefCell, fmt::Display, cmp, sync::{Arc, Mutex}};

#[derive(Debug, Clone)]
pub enum NodeType {
    Sync(Arc<Mutex<DawgNode>>),
    Unsync(Rc<RefCell<DawgNode>>),
}

impl NodeType {
    pub fn get_unsync(&self) -> Option<&Rc<RefCell<DawgNode>>> {
        match self {
            NodeType::Unsync(node) => {Some(node)}
            NodeType::Sync(_) => {None}
        }
    }

    pub fn get_sync(&self) -> Option<&Arc<Mutex<DawgNode>>> {
        match self {
            NodeType::Unsync(_) => {None}
            NodeType::Sync(node) => {Some(node)}
        }
    }
}



#[derive(Debug)]
pub struct DawgNode {
    /// id of the node
    pub(crate) id: usize,
    /// value is true if this node is the end of a word
    pub(crate) terminal: bool,
    /// Returns all the other nodes (e.g, letters) extending from this node (letter)
    pub(crate) edges: HashMap<String, NodeType>,
    /// returns the number of words so far that have been formed from the root of the dawg up to this node
    pub(crate) count: usize,
}

impl DawgNode {
    pub fn new(id: usize) -> Self {
        Self { id, terminal: false, edges: HashMap::new(), count: 0 }
    }

    pub(crate) fn num_reachable(&mut self) -> usize {
        if self.count != 0 {
            return self.count;
        }

        let mut count = 0;

        if self.terminal {
            count += 1;
        }

        for (_, value) in &mut self.edges {
            match value {
                NodeType::Unsync(node) => {
                    if let Some(pre_value) = Rc::get_mut(node) {
                        count += pre_value.get_mut().num_reachable();
                    }
                }
                NodeType::Sync(node) => {
                    if let Some(pre_value) = Arc::get_mut(node) {
                        // count += pre_value.lock().unwrap().num_reachable();
                        count += pre_value.get_mut().unwrap().num_reachable();
                    }
                }
            }
        }

        self.count = count;
        return count;
    }
}

impl Display for DawgNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut arr = vec![];
        if self.terminal {
            arr.push(String::from("1"));
        } else {
            arr.push(String::from("0"));
        }

        for (key, value) in &self.edges {

            let id = match value {
                NodeType::Sync(node) => node.lock().unwrap().id.to_string(),
                NodeType::Unsync(node) => node.try_borrow().unwrap().id.to_string(),
            };
            
            arr.push(id);
            arr.push(key.to_owned());
        }

        let name = arr.join("_");
        write!(f, "{}", name)
    }
}

impl Ord for DawgNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl PartialOrd for DawgNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))       
    }
}

impl PartialEq for DawgNode {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DawgNode {}




#[derive(Debug, Clone)]
pub(crate) struct TriDawg {
    pub(crate) parent: NodeType,
    pub(crate) letter: String,
    pub(crate) child: NodeType,
}

impl TriDawg {
    pub fn new(parent: NodeType, letter: String, child: NodeType) -> Self {
        Self { parent, letter, child, }
    }
}


pub(crate) enum SearchReq {
    Vertex,
    Word,
}


#[derive(Debug)]
pub(crate) struct SearchRes<T> {
    pub(crate) node: T,
    pub(crate) word: String
}

impl<T> SearchRes<T> {
    pub fn new(node: T, word: String) -> Self {
        Self { node, word }
    }
}

pub trait Wrapper {
    fn new() -> Self;

    fn create(&mut self) -> NodeType;
}


#[derive(Debug)]
pub struct Dawg<T: Wrapper> {
    pub(crate) node: T,
    pub(crate) minimized_nodes: HashMap<String, NodeType>,
    pub(crate) root: NodeType,
    pub(crate) unchecked_nodes: Vec<TriDawg>,
    pub(crate) previous_word: String,
}


#[derive(Debug, Clone)]
pub(crate) struct DawgWrapper {
    next_id: usize,
}

impl Wrapper for DawgWrapper {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    fn create(&mut self) -> NodeType {
        let node = DawgNode::new(self.next_id);
        self.next_id += 1;
        NodeType::Unsync(Rc::new(RefCell::new(node)))
    }
}



// impl<D> Dawg<D> where D: Wrapper {
//     fn make() -> Self {
//         let mut dawg_wrapper = DawgWrapper::new();
//         // let result = dawg_wrapper.create();
//         Self {
//             root: dawg_wrapper.create(),
//             minimized_nodes: HashMap::new(),
//             unchecked_nodes: vec![],
//             previous_word: String::from(""),
//             node: dawg_wrapper,
//         }
//     }
// }