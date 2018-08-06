//! Everything related to Uavcan Nodes

use lib::core::marker::PhantomData;

use {
    Frame,
    Struct,
    Message,
};

use storage::{
    Storage,
    SubscriberStorageHandle,
    InterfaceStorageHandle,
};

use transfer::{
    TransferInterface,
    TransferFrame,
    TransferFrameID,
    TransferID,
    TransferFrameIDFilter,
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
pub trait Node<I: TransferInterface, S: Storage<I::Frame>> {

    /// Broadcast a `Message` on the Uavcan network. 
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError>;

    /// Subscribe to broadcasts of a specific `Message`.
    fn subscribe<T: Struct + Message>(&self) -> Subscriber<T, I::Frame, S::SubscriberStorageHandle>;
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
pub struct Subscriber<T: Struct + Message, F: TransferFrame, H: SubscriberStorageHandle<F>> {
    storage_handle: H,
    phantom: PhantomData<(T, F)>,
}

impl <T: Struct + Message, F: TransferFrame, H: SubscriberStorageHandle<F>> Subscriber<T, F, H> {
    fn new(storage_handle: H) -> Self {
        Subscriber{
            storage_handle,
            phantom: PhantomData,
        }
    }

    /// Receives a message that is subscribed on.
    ///
    /// Messages are returned in a manner that respects the `TransferFrameID` priority.
    /// For equal priority, FIFO logic is used.
    pub fn receive(&self) -> Option<Result<T, ReceiveError>> {
        if let Some(full_id) = self.storage_handle.find_id(|x| x.is_end_frame()) {
            let mut assembler = FrameAssembler::new();
            loop {
                match assembler.add_transfer_frame(self.storage_handle.remove(&full_id.frame_id).unwrap()) {
                    Err(AssemblerError::ToggleError) => {
                        self.storage_handle.retain(|x| x.full_id() != full_id);
                        return Some(Err(ReceiveError {
                            transfer_frame_id: full_id.frame_id,
                            transfer_id: full_id.transfer_id,
                            error_code: ReceiveErrorCode::ToggleError,
                        }));
                    },
                    Err(_) => panic!("Unexpected error from FrameAssembler"),
                    Ok(AssemblerResult::Finished) => {
                        match assembler.build() {
                            Ok(frame) => return Some(Ok(frame.into_parts().1)),
                            Err(BuildError::CRCError) => {
                                self.storage_handle.retain(|x| x.full_id() != full_id);
                                return Some(Err(ReceiveError {
                                    transfer_frame_id: full_id.frame_id,
                                    transfer_id: full_id.transfer_id,
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
pub struct SimpleNode<I, D, S>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I>,
          S: Storage<I::Frame>,
{
    interface: D,
    interface_storage: S::InterfaceStorageHandle,
    storage: S,
    config: NodeConfig,
}


impl<I, D, S> SimpleNode<I, D, S>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I>,
          S: Storage<I::Frame>,
{
    pub fn new(interface: D, config: NodeConfig) -> Self {
        let storage = S::new();
        SimpleNode{
            interface: interface,
            interface_storage: storage.new_interface(),
            config: config,
            storage: storage,
        }
    }

    /// Call this method after the interface have sucesfully received a new frame or periodically
    ///
    /// This method is responsible for moving as many frames as possible
    /// from incoming interface mailboxes to the storage buffer.
    pub fn flush_receptions(&self) {
        while let Some(new_frame) = self.interface.receive() {
            self.storage.insert_subscriber_queue(new_frame).expect("Storage full");
        }
    }

    /// Call this method after the interface have successfully transmitted a new frame or periodically
    ///
    /// This method is responsible for moving as many frames as possible
    /// from storage buffers to the outgoing interface mailboxes.
    pub fn flush_transmissions(&self) {
        //TODO: Handle priority inversion concerns correctly
        while let Some(top_frame) = self.interface_storage.pop() {
            match self.interface.transmit(&top_frame) {
                Ok(_) => (),
                Err(_) => {
                    self.interface_storage.push(top_frame).expect("Storage Full");
                    return;
                }
            }
        }
    }
}


impl<I, D, S> Node<I, S> for SimpleNode<I, D, S>
    where I: TransferInterface,
          D: ::lib::core::ops::Deref<Target=I>,
          S: Storage<I::Frame>,
{
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError> {
        let priority = 0;
        let transfer_id = TransferID::new(0);
        
        let mut generator = if let Some(ref node_id) = self.config.id {
            FrameDisassembler::from_uavcan_frame(Frame::from_message(message, priority, *node_id), transfer_id)
        } else {
            unimplemented!("Anonymous transfers not implemented")
        };
        
        while let Some(can_frame) = generator.next_transfer_frame() {
            self.storage.insert_interface_queue(can_frame).unwrap();
        }
        // TODO: Transfer into interface at this point or first attempt to add directly to interface.

        Ok(())
    }

    fn subscribe<T: Struct + Message>(&self) -> Subscriber<T, I::Frame, S::SubscriberStorageHandle> {
        let id = if let Some(type_id) = T::TYPE_ID {
            u32::from(type_id) << 8
        } else {
            unimplemented!("Resolvation of type id is not supported yet")
        };

        let filter = TransferFrameIDFilter::new(id, 0x1ff << 7);
    
        Subscriber::new(self.storage.subscribe_to(filter))
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
