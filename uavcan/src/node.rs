use lib::core::marker::PhantomData;

use {
    Frame,
    Struct,
    Message,
};

use transfer::{
    TransferInterface,
    TransferID,
    FullTransferID,
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
    }
}

pub trait Node<'a, I: TransferInterface<'a>> {
    fn transmit_message<T: Struct + Message>(&self, message: T) -> Result<(), IOError>;
    fn receive_message<T: Struct + Message>(&self) -> Result<T, IOError>;
}

    

pub struct NodeConfig {
    pub id: Option<NodeID>,
}

pub struct SimpleNode<'a, I>
    where I: TransferInterface<'a> + 'a {
    interface: I,
    config: NodeConfig,
    phantom: PhantomData<&'a I>,
}


impl<'a, I> SimpleNode<'a, I>
    where I: TransferInterface<'a> + 'a {
    pub fn new(interface: I, config: NodeConfig) -> Self {
        SimpleNode{
            interface: interface,
            config: config,
            phantom: PhantomData,
        }
    }
}

impl<'a, I> Node<'a, I> for SimpleNode<'a, I>
    where I: TransferInterface<'a> + 'a {
    
    fn transmit_message<T: Struct + Message>(&self, message: T) -> Result<(), IOError> {
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

    fn receive_message<T: Struct + Message>(&self) -> Result<T, IOError> {
        let identifier = FullTransferID {
            frame_id: T::id(0, NodeID::new(0)),
            transfer_id: TransferID::new(0),
        };
        let mask = FullTransferID {
            frame_id: T::id(0, NodeID::new(0)),
            transfer_id: TransferID::new(0),
        };

        if let Some(id) = self.interface.completed_receive(identifier, mask) {
            let mut assembler = FrameAssembler::new();
            loop {
                match assembler.add_transfer_frame(self.interface.receive(&id).unwrap()) {
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
