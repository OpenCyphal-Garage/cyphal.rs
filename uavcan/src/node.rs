use lib::core::marker::PhantomData;

use {
    Frame,
    Struct,
    Message,
};

use transfer::{
    TransferInterface,
    TransferFrame,
    TransferID,
    TransferFrameIDFilter,
    TransferSubscriber,
};

use frame_disassembler::FrameDisassembler;
use frame_assembler::FrameAssembler;
use frame_assembler::AssemblerResult;

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
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError>;
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
    pub fn receive(&self) -> Result<T, IOError> {
        // TODO: mind the priority!
        if let Some(end_frame) = self.transfer_subscriber.find(|x| x.is_end_frame()) {
            let mut assembler = FrameAssembler::new();
            loop {
                match assembler.add_transfer_frame(self.transfer_subscriber.receive(&end_frame.id()).unwrap()) {
                    Err(_) => return Err(IOError::Other), // fix error message
                    Ok(AssemblerResult::Finished) => break,
                    Ok(AssemblerResult::Ok) => (),
                }
            }
            Ok(assembler.build().unwrap().into_parts().1)
        } else {
            Err(IOError::Other) // fix error message
        }
    }
    
    
}


/// A minimal featured Uavcan node
///
/// Supports the features required by `Node` trait
#[derive(Debug)]
pub struct SimpleNode<'a, I>
    where I: 'a + TransferInterface {
    interface: &'a I,
    config: NodeConfig,
    phantom: PhantomData<I>,
}


impl<'a, I> SimpleNode<'a, I>
    where I: 'a + TransferInterface {
    pub fn new(interface: &'a I, config: NodeConfig) -> Self {
        SimpleNode{
            interface: interface,
            config: config,
            phantom: PhantomData,
        }
    }
}


impl<'a, I> Node<I> for SimpleNode<'a, I>
    where I: 'a + TransferInterface {
    fn broadcast<T: Struct + Message>(&self, message: T) -> Result<(), IOError> {
        let priority = 0;
        let transfer_id = TransferID::new(0);
        
        let mut generator = if let Some(ref node_id) = self.config.id {
            FrameDisassembler::from_uavcan_frame(Frame::from_message(message, priority, *node_id), transfer_id)
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
