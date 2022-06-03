use std::{sync::{Arc, Mutex}, path, collections::HashMap};
use std::cmp;

// pub type Node = Arc<Mutex<DawgNode>>;
// // use crate::dawg::unsync::DawgNode;


// // #[derive(Debug)]
// // pub struct DawgNode {
// //     /// id of the node
// //     id: usize,
// //     /// value is true if this node is the end of a word
// //     terminal: bool,
// //     /// Returns all the other nodes (e.g, letters) extending from this node (letter)
// //     edges: HashMap<String, Node>,
// //     /// returns the number of words so far that have been formed from the root of the dawg up to this node
// //     count: usize,
// // }



// impl DawgNode {
//     pub fn top(&self) {
//         let obc = &self.terminal;
//     }
// }
