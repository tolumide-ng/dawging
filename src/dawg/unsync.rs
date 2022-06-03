use std::{collections::HashMap, rc::Rc, cell::RefCell, cmp};

use crate::utils::Utils;

use super::{common::{NodeType, Wrapper, DawgNode, Dawg, TriDawg, SearchReq, SearchRes}};


/// Wrapper for DawgNode to persist the next_id of the DawgNode that would be added to the Dawg
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


impl<T> Dawg<T> where T: Wrapper {
    pub fn new() -> Dawg<impl Wrapper> {
        let mut d_w = DawgWrapper::new();
        
        Dawg { 
            root: d_w.create(),
            node: d_w,
            minimized_nodes: HashMap::new(),
            unchecked_nodes: vec![],
            previous_word: String::new(),
        }
    }

    pub fn minimize(&mut self, down_to: usize) {
        let mut start = self.unchecked_nodes.len() as i8 - 1;
        let end = down_to as i8 - 1;

        while start > end {
            let i = start as usize;
            let TriDawg { parent, letter, child } = &mut self.unchecked_nodes[i];
            let parent = parent.get_unsync().unwrap();
            let child = child.get_unsync().unwrap();
            let node = child.try_borrow().unwrap().to_string();

            let exists = self.minimized_nodes.contains_key(node.as_str());

            if exists {
                let minimized_reference = self.minimized_nodes.get(node.as_str()).unwrap().get_unsync().unwrap();
                parent.as_ref().borrow_mut().edges.insert(letter.to_owned(), NodeType::Unsync(Rc::clone(minimized_reference)));
            } else {
                self.minimized_nodes.insert(node, NodeType::Unsync(Rc::clone(child)));
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
        let word_vec = Utils::split_to_vec(word.to_owned());
        let prev_word_vec = Utils::split_to_vec(self.previous_word.to_owned());

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
            
            if  self.unchecked_nodes.len() != 0 {
                let last = self.unchecked_nodes.len() -1;
                node = &self.unchecked_nodes[last].child;
            }
            
            let next_node = self.node.create();
            node.get_unsync().unwrap().as_ref().borrow_mut().edges.insert(letter.to_owned(), NodeType::Unsync(Rc::clone(&next_node.get_unsync().unwrap())));

            let tridawg = TriDawg::new(NodeType::Unsync(Rc::clone(node.get_unsync().unwrap())), letter, NodeType::Unsync(Rc::clone(next_node.get_unsync().unwrap())));
            self.unchecked_nodes.push(tridawg);
        }

        let last_unchecked = self.unchecked_nodes.len() -1;
        let node = &mut self.unchecked_nodes[last_unchecked].child.get_unsync().unwrap().as_ref().borrow_mut();
        node.terminal = true;
        self.previous_word = word;
    }

    pub fn finish(&mut self) {
        self.minimize(0);
        self.root.get_unsync().unwrap().as_ref().borrow_mut().num_reachable();
        self.minimized_nodes = HashMap::new();
        self.unchecked_nodes = vec![];
    }

    fn find(&self, word: &String, return_type: SearchReq, case_sensitive: bool) -> Option<SearchRes<Rc<RefCell<DawgNode>>>> {
        let mut node = Rc::clone(&self.root.get_unsync().unwrap());
        let word_vec = Utils::split_to_vec(word.to_owned());

        for i in 0..word.len() {
            let letter = word_vec[i].to_string();
            let keys = node.as_ref().borrow().edges.keys().collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();

            match case_sensitive {
                true => {
                    if keys.contains(&letter) {
                        // let nnnode = ;
                        let next_node = Rc::clone(node.as_ref().borrow().edges[&letter].get_unsync().unwrap());
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
                        let next_node = Rc::clone(&node.as_ref().borrow().edges[&actual_key].get_unsync().unwrap());
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
                return Some(context.word)
            }
        }

        None
    }

    /// find out if word is a prefix of anything in the dictionary
    pub fn lookup(&self, word: String, case_sensitive: bool) -> Option<Rc<RefCell<DawgNode>>> {
        if let Some(context) = self.find(&word, SearchReq::Vertex, case_sensitive) {
            if context.node.as_ref().borrow().terminal {
                return Some(context.node)
            }
        }
        None
    }


}
