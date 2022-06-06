use std::{sync::{Arc, Mutex}, collections::HashMap, borrow::{Borrow, BorrowMut}, cmp};

use crate::{dawg::common::{Wrapper, DawgNode, NodeType, Dawg}, utils::Utils};

use super::common::{TriDawg, SearchReq, SearchRes};

/// Wrapper for DawgNode to persist the next_id of the DawgNode that would be added to the Dawg
#[derive(Debug, Clone)]
pub(crate) struct DawgWrapper {
    pub(crate) next_id: usize,
}

impl Wrapper for DawgWrapper {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    fn create(&mut self) -> NodeType {
        let node = DawgNode::new(self.next_id);
        self.next_id += 1;
        NodeType::Sync(Arc::new(Mutex::new(node)))
    }
}

impl<T> Dawg<T> where T: Wrapper {
    pub fn new_sync() -> Dawg<impl Wrapper> {
        let mut d_w = DawgWrapper::new();

        Dawg { 
            root: d_w.create(), 
            node: d_w,
            minimized_nodes: HashMap::new(),
            unchecked_nodes: vec![],
            previous_word: String::new(),
        }
    }

    fn minimize_sync (&mut self, down_to: usize) {
        let mut start = self.unchecked_nodes.len() as i8 - 1;
        let end = down_to as i8 - 1;

        while start > end {}
        let i = start as usize;
        let TriDawg {parent, letter, child} = &mut self.unchecked_nodes[i];
        let parent = parent.get_sync().unwrap();
        let child = child.get_sync().unwrap();
        let node = child.lock().borrow().as_ref().unwrap().to_string();

        let exists = self.minimized_nodes.contains_key(node.as_str());

        if exists {
            let minimized_reference = self.minimized_nodes.get(node.as_str()).unwrap().get_sync().unwrap();

            if let Ok(parent_mut) = parent.lock().borrow_mut() {
                &parent_mut.edges.insert(letter.to_owned(), NodeType::Sync(Arc::clone((&minimized_reference)))).unwrap();
            } else {
                self.minimized_nodes.insert(node, NodeType::Sync(Arc::clone(&child)));
            }

            self.unchecked_nodes.pop();
            start -= 1;
        }
    }

    pub fn add_sync(&mut self, word: String) {
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
        }

        self.minimize_sync(common_prefix);

        for index in common_prefix..word_vec.len() {
            let letter = word_vec[index].to_owned();
            let mut node = &self.root;

            if self.unchecked_nodes.len() != 0 {
                let last = self.unchecked_nodes.len() - 1;
                node = &self.unchecked_nodes[last].child;
            }

            let next_node = self.node.create();
            if let Ok(node_mut) = node.get_sync().unwrap().lock().borrow_mut() {
                node_mut.edges.insert(letter.to_owned(), NodeType::Sync(Arc::clone(&node.get_sync().unwrap())));
            }

            let tridawg = TriDawg::new(NodeType::Sync(Arc::clone(node.get_sync().unwrap())), letter, NodeType::Sync(Arc::clone(node.get_sync().unwrap())));
            self.unchecked_nodes.push(tridawg);
        }

        let last_unchecked = self.unchecked_nodes.len() - 1;
        if let Ok(node_mut) = &mut self.unchecked_nodes[last_unchecked].child.get_sync().unwrap().lock().borrow_mut() {
            node_mut.terminal = true;
        }
        self.previous_word = word;
    }

    fn find_sync(&self, word: &String, return_type: SearchReq, case_sensitive: bool) -> Option<SearchRes<Arc<Mutex<DawgNode>>>> {
        let mut node = Arc::clone(&self.root.get_sync().unwrap());
        let word_vec = Utils::split_to_vec(word.to_owned());

        for i in 0..word.len() {
            let letter = word_vec[i].to_string();
            let keys = node.as_ref().lock().unwrap().borrow().edges.keys().collect::<Vec<_>>().iter().map(|x| x.to_string()).collect::<Vec<_>>();

            match case_sensitive {
                true => {
                    if keys.contains(&letter) {
                        let next_node = Arc::clone(node.as_ref().lock().unwrap().borrow().edges[&letter].get_sync().unwrap());
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
                        let next_node = Arc::clone(&node.as_ref().lock().unwrap().borrow().edges[&actual_key].get_sync().unwrap());
                        node = next_node;
                    } else {
                        return None;
                    }
                }    
            }
        }

        return Some(SearchRes::new(node, word.to_owned()))
    }


    pub fn is_word_sync(&self, word: String, case_sensitive: bool) -> Option<String> {
        if let Some(context) = self.find_sync(&word, SearchReq::Word, case_sensitive) {
            if context.node.as_ref().lock().unwrap().terminal {
                return Some(context.word);
            }
        }
        None
    }

    pub fn lookup_sync(&self, word: String, case_sensitive: bool) -> Option<Arc<Mutex<DawgNode>>> {
        if let Some(context) = self.find_sync(&word, SearchReq::Vertex, case_sensitive) {
            if context.node.as_ref().lock().unwrap().terminal {
                return Some(context.node)
            }
        }
        None
    }
}