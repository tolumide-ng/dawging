use std::{collections::HashMap, rc::Rc, cell::RefCell, fmt::Display, borrow::Cow};

type Node = Rc<RefCell<DawgNode>>;

#[derive(Debug, Eq)]
struct DawgNode {
    /// id of the node
    id: usize,
    /// value is true if this node is the end of a word
    terminal: bool,
    /// Returns all the other nodes (e.g, letters) extending from this node (letter)
    edges: HashMap<String, Node>,
    /// returns the number of words so far that have been formed from the root of the dawg up to this node
    count: usize,
}

/// Wrapper for DawgNode to persist the next_id of the DawgNode that would be added to the Dawg
#[derive(Debug, Clone)]
struct DawgWrapper {
    next_id: usize,
}

impl DawgWrapper {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    fn create(&mut self) -> Node {
        let node = DawgNode::new(self.next_id);
        self.next_id += 1;
        Rc::new(RefCell::new(node))
    }
}


impl DawgNode {
    fn new(id: usize) -> Self {
        Self { id, terminal: false, edges: HashMap::new(), count: 0 }
    }

    fn num_reachable(&mut self) -> usize {
        if self.count != 0 {
            return self.count;
        }

        let mut count = 0;

        if self.terminal {
            count += 1;
        }

        for (_, value) in &mut self.edges {
            if let Some(pre_value) = Rc::get_mut(value) {
                count += pre_value.get_mut().num_reachable();
            }
        }

        self.count = 0;
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
            let id = value.try_borrow().unwrap().id.to_string();
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

#[derive(Debug, Clone)]
pub struct TriDawg {
    parent: Node,
    letter: String,
    child: Node,
}

impl TriDawg {
    fn new(parent: Node, letter: String, child: Node) -> Self {
        Self { parent, letter, child, }
    }
}


enum SearchReq {
    Vertex,
    Word,
}


#[derive(Debug)]
struct SearchRes<'a> {
    node: Node,
    word: Cow<'a, str>
}

impl<'a> SearchRes<'a> {
    pub fn new(node: Node, word: Cow<'a, str>) -> Self {
        Self { node, word }
    }
}