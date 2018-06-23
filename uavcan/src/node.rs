//! Everything related to Uavcan Nodes

use lib::core::marker::PhantomData;

use {
    Frame,
    Struct,
    Message,
};

use versioning::{
    ProtocolVersion,
};

use transfer::{
    TransferInterface,
    TransferFrame,
    TransferFrameID,
    TransferID,
    TransferFrameIDFilter,
    TransferSubscriber,
};

use frame_disassembler::FrameDisassembler;
use frame_assembler::FrameAssembler;
use frame_assembler::AssemblerResult;
use frame_assembler::AssemblerError;
use frame_assembler::BuildError;

use embedded_types::io::Error as IOError;

/// The 7 bit `NodeID` used in Uavcan
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NodeID(u8);

impl NodeID {
    /// Creates a new NodeID
    ///
    /// # Examples
    /// ```
    ///
    /// use uavcan::NodeID;
    ///
    /// let node_id = NodeID::new(127);
    ///
    /// ```
    ///
    /// # Panics
    /// Panics if `id > 127` or `id == 0`
    pub fn new(id: u8) -> NodeID {
        assert_ne!(id, 0, "Uavcan node IDs can't be 0");
        assert!(id <= 127, "Uavcan node IDs must be 7bit (<127)");
        NodeID(id)
    }}


/// The Uavcan node trait.
///
/// Allows implementation of application level features genericaly for all types of Uavcan Nodes.
pub trait Node<I: TransferInterface> {

    /// Broadcast a `Message` on the Uavcan network. 
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError>;

    /// Subscribe to broadcasts of a specific `Message`.
    fn subscribe<T: Struct + Message>(&self) -> Result<Subscriber<T, I>, ()>;
}

    
/// Configuration for an `Node`
///
/// # Examples
/// ```
///
/// use uavcan::NodeConfig;
/// use uavcan::NodeID;
///
/// let mut node_config = NodeConfig::default();
/// node_config.id = Some(NodeID::new(127));
///
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeConfig {

    /// An optional Uavcan `NodeId`
    ///
    /// Nodes with `id = None` is, in Uavcan terms, an anonymous Node.
    pub id: Option<NodeID>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig{
            id: None,
        }
    }
}


/// A subscription handle used to receive a specific `Message`
#[derive(Debug)]
pub struct Subscriber<T: Struct + Message, I: TransferInterface> {
    transfer_subscriber: I::Subscriber,
    phantom: PhantomData<T>,
}

impl <T: Struct + Message, I: TransferInterface> Subscriber<T, I> {
    fn new(transfer_subscriber: I::Subscriber) -> Self {
        Subscriber{
            transfer_subscriber: transfer_subscriber,
            phantom: PhantomData,
        }
    }

    /// Receives a message that is subscribed on.
    ///
    /// Messages are returned in a manner that respects the `TransferFrameID` priority.
    /// For equal priority, FIFO logic is used.
    pub fn receive(&self) -> Option<Result<T, ReceiveError>> {
        if let Some(end_frame) = self.transfer_subscriber.find(|x| x.is_end_frame()) {
            let mut assembler = FrameAssembler::new();
            loop {
                match assembler.add_transfer_frame(self.transfer_subscriber.receive(&end_frame.id()).unwrap()) {
                    Err(AssemblerError::ToggleError) => {
                        self.transfer_subscriber.retain(|x| x.full_id() != end_frame.full_id());
                        return Some(Err(ReceiveError {
                            transfer_frame_id: end_frame.id(),
                            transfer_id: end_frame.tail_byte().transfer_id(),
                            error_code: ReceiveErrorCode::ToggleError,
                        }));
                    },
                    Err(_) => panic!("Unexpected error from FrameAssembler"),
                    Ok(AssemblerResult::Finished) => {
                        match assembler.build() {
                            Ok(frame) => return Some(Ok(frame.into_parts().1)),
                            Err(BuildError::CRCError) => {
                                self.transfer_subscriber.retain(|x| x.full_id() != end_frame.full_id());
                                return Some(Err(ReceiveError {
                                    transfer_frame_id: end_frame.id(),
                                    transfer_id: end_frame.tail_byte().transfer_id(),
                                    error_code: ReceiveErrorCode::CRCError,
                                }));
                            },
                            Err(_) => panic!("Unexpected error from FrameAssembler"),
                        }
                    },
                    Ok(AssemblerResult::Ok) => (),
                }
            }
        } else {
            None
        }
    }
    
    
}

/// Full Error status from a failed receive
#[derive(Debug, PartialEq, Eq)]
pub struct ReceiveError {
    pub transfer_frame_id: TransferFrameID,
    pub transfer_id: TransferID,
    pub error_code: ReceiveErrorCode,
}

/// The error kind for a failed receive
#[derive(Debug, PartialEq, Eq)]
pub enum ReceiveErrorCode {
    CRCError,
    ToggleError,
}

/// A minimal featured Uavcan node.
///
/// This type of node lack some features that the `FullNode` provides,
/// but is in turn suitable for highly resource constrained systems.
#[derive(Debug)]
pub struct SimpleNode<I, D>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I> {
    interface: D,
    config: NodeConfig,
}


impl<I, D> SimpleNode<I, D>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I> {
    pub fn new(interface: D, config: NodeConfig) -> Self {
        SimpleNode{
            interface: interface,
            config: config,
        }
    }
}


impl<I, D> Node<I> for SimpleNode<I, D>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I> {
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError> {
        let priority = 0;
        let transfer_id = TransferID::new(0);
        
        let mut generator = if let Some(ref node_id) = self.config.id {
            FrameDisassembler::from_uavcan_frame(Frame::from_message(message, priority, ProtocolVersion::Version0, *node_id), transfer_id)
        } else {
            unimplemented!("Anonymous transfers not implemented")
        };
        
        while let Some(can_frame) = generator.next_transfer_frame() {
            self.interface.transmit(&can_frame)?;
        }

        Ok(())
    }

    fn subscribe<T: Struct + Message>(&self) -> Result<Subscriber<T, I>, ()> {
        let id = if let Some(type_id) = T::TYPE_ID {
            u32::from(type_id) << 8
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        };

        let filter = TransferFrameIDFilter::new(id, 0x1ff << 7);
    
        Ok(Subscriber::new(self.interface.subscribe(filter)?))
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
