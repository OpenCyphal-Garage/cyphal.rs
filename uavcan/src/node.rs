pub struct NodeID(u8);

impl NodeID {
    /// Creates a new NodeID
    ///
    /// # Panics
    /// Panics if `id > 127`
    pub fn new(id: u8) -> NodeID {
        assert!(id <= 127, "Uavcan node IDs must be 7bit");
        NodeID(id)
    }
}


impl From<NodeID> for u8 {
    fn from(id: NodeID) -> u8 {
        id.0
    }
}

impl From<NodeID> for u32 {
    fn from(id: NodeID) -> u32 {
        u32::from(id.0)
    }
}
