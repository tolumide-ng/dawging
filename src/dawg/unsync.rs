use std::{collections::HashMap, rc::Rc, cell::RefCell, fmt::Display, cmp};

pub type Node = Rc<RefCell<DawgNode>>;

#[derive(Debug, Eq)]
pub struct DawgNode {
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
struct SearchRes {
    node: Node,
    word: String
}

impl SearchRes {
    pub fn new(node: Node, word: String) -> Self {
        Self { node, word }
    }
}

#[derive(Debug)]
pub struct Dawg {
    root: Node,
    minimized_nodes: HashMap<String, Node>,
    unchecked_nodes: Vec<TriDawg>,
    previous_word: String,
    node: DawgWrapper,
}


impl Dawg {
    pub fn new() -> Self {
        let mut dawg_wrapper = DawgWrapper::new();
        Self { 
            root: dawg_wrapper.create(), 
            minimized_nodes: HashMap::new(), 
            unchecked_nodes: vec![], 
            previous_word: String::from(""), 
            node: dawg_wrapper
        }
    }

    pub fn minimize(&mut self, down_to: usize) {
        let mut start = self.unchecked_nodes.len() as i8 - 1;
        let end = down_to as i8 - 1;

        while  start > end  {
            let i = start as usize;
            let TriDawg { parent, letter, child } = &mut self.unchecked_nodes[i];
            let node = child.try_borrow().unwrap().to_string();
            let exists = self.minimized_nodes.contains_key(node.as_str());

            if exists {
                let minimized_reference = self.minimized_nodes.get(node.as_str()).unwrap();
                parent.as_ref().borrow_mut().edges.insert(letter.to_owned(), Rc::clone(minimized_reference));
            } else {
                self.minimized_nodes.insert(node, Rc::clone(child));
            }

            self.unchecked_nodes.pop();
            start -= 1;
        }
    }

    pub fn add(&mut self, word: String) {
        if self.previous_word > word {
            panic!("Error: Please ensure all words are sorted bedore adding");
        }

        let mut common_prefix = 0;

        let word_vec = word.split_terminator("").skip(1).collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let prev_word_vec = self.previous_word.split_terminator("").skip(1).collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();

        let min_length = cmp::min(word_vec.len(), prev_word_vec.len());

        for index in 0..min_length {
            if word_vec[index] != prev_word_vec[index] {
                break;
            }
            common_prefix += 1;
        }

        self.minimize(common_prefix);

        for index in common_prefix..word_vec.len() {
            let letter = word_vec[index].to_owned();
            let mut node = &self.root;

            if self.unchecked_nodes.len() != 0 {
                let last = self.unchecked_nodes.len()-1;
                node = &self.unchecked_nodes[last].child;
            }

            let next_node = self.node.create();
            node.as_ref().borrow_mut().edges.insert(letter.to_owned(), Rc::clone(&next_node));

            let tridawg = TriDawg::new(Rc::clone(node), letter, Rc::clone(&next_node));
            self.unchecked_nodes.push(tridawg);
        }

        let last_unchecked = self.unchecked_nodes.len() -1;
        let node = &mut self.unchecked_nodes[last_unchecked].child.as_ref().borrow_mut();
        node.terminal = true;
        self.previous_word = word;
    }

    pub fn finish(&mut self) {
        self.minimize(0);
        self.root.as_ref().borrow_mut().num_reachable();
        self.minimized_nodes = HashMap::new();
        self.unchecked_nodes = vec![];
    }

    fn find(&self, word: &String, return_type: SearchReq, case_sensitive: bool) -> Option<SearchRes> {
        let mut node: Node = Rc::clone(&self.root);
        let word_vec = word.split_terminator("").skip(1).collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();

        for i in 0..word.len() {
            let letter = word_vec[i].to_string();
            let keys = node.as_ref().borrow().edges.keys().collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();


            match case_sensitive {
                true => {
                    if keys.contains(&letter) {
                        let next_node = Rc::clone(&node.as_ref().borrow().edges[&letter]);
                        node = next_node;
                    } else {
                        return None;
                    }
                }
                false => {
                    let modified_keys = keys.iter().map(|x| x.to_uppercase()).collect::<Vec<_>>();
                    let letter = letter.to_uppercase();

                    if let Some(index) = modified_keys.iter().position(|x| x == &letter) {
                        let actual_key = keys[index].to_owned();
                        let next_node = Rc::clone(&node.as_ref().borrow().edges[&actual_key]);
                        node = next_node;
                    } else {
                        return None;
                    }
                }
            }
        }
        return Some(SearchRes::new(node, word.to_owned()))
    }

    /// Given a specific word, check if the word exists in the lexicon (Allowing search to be case sensitive or insensitive)
    pub fn is_word(&self, word: String, case_sensitive: bool) -> Option<String> {
        if let Some(context) = self.find(&word, SearchReq::Word, case_sensitive) {
            if context.node.as_ref().borrow().terminal {
                return Some( context.word)
            }
        }

        None
    }

    /// find out if word is a prefix of anything in the dictionary
    pub fn lookup(&self, word: &String, case_sensitive: bool) -> Option<Node> {
        if let Some(context) = self.find(&word, SearchReq::Vertex, case_sensitive) {
            if context.node.as_ref().borrow().terminal {
                return Some(context.node)
            }
        }
        None
    }
}