use ids::*;
use traversal::*;
use connectivity_info::ConnectivityInfo;
use std::rc::Rc;

pub type VertexIterator = Box<Iterator<Item = VertexID>>;
