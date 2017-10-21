#[macro_export]
macro_rules! message_frame_header{
    ($t:ident, $n:expr) => (
        #[derive(Debug, PartialEq, Default)]
        struct $t {
            priority: u8,
            source_node: u8,
        }        
        
        impl uavcan::Header for $t {
            fn id(&self) -> uavcan::transfer::TransferFrameID {
                let mut id = 0;
                id.set_bits(0..7, self.source_node as u32);
                id.set_bit(7, false);
                id.set_bits(8..24, <Self as uavcan::MessageFrameHeader>::TYPE_ID as u32);
                id.set_bits(24..29, self.priority as u32);
                uavcan::transfer::TransferFrameID::new(id)
            }
            
            fn from_id(id: uavcan::transfer::TransferFrameID) -> Result<Self, ()> {
                if u32::from(id).get_bits(8..24) != <Self as uavcan::MessageFrameHeader>::TYPE_ID as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: u32::from(id).get_bits(24..29) as u8,
                        source_node: u32::from(id).get_bits(0..7) as u8,
                    })
                }
            }
            fn set_priority(&mut self, priority: u8) {
                self.priority.set_bits(0..5, priority);
            }
            fn get_priority(&self) -> u8 {
                self.priority.get_bits(0..5)
            }
        }
        
        impl uavcan::MessageFrameHeader for $t {
            const TYPE_ID: u16 = $n;
            
            fn new(priority: u8, source_node: u8) -> Self {
                Self{
                    priority: priority,
                    source_node: source_node,                    
                }
            }
        }
    );
}

#[macro_export]
macro_rules! anonymous_frame_header{
    ($t:ident, $n:expr) => (
        #[derive(Debug, PartialEq, Default)]
        struct $t {
            priority: u8,
            discriminator: u16,
        }

        impl uavcan::Header for $t {
            fn id(&self) -> uavcan::transfer::TransferFrameID {
                let mut id = 0;
                id.set_bits(0..7, 0);
                id.set_bit(7, false);
                id.set_bits(8..10, <Self as uavcan::AnonymousFrameHeader>::TYPE_ID as u32);
                id.set_bits(10..24, self.discriminator as u32);
                id.set_bits(24..29, self.priority as u32);
                uavcan::transfer::TransferFrameID::new(id)
            }
   
            fn from_id(id: uavcan::transfer::TransferFrameID) -> Result<Self, ()> {
                if u32::from(id).get_bits(8..24) != <Self as uavcan::AnonymousFrameHeader>::TYPE_ID as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: u32::from(id).get_bits(24..29) as u8,
                        discriminator: u32::from(id).get_bits(10..24) as u16,
                    })
                }
            }
            fn set_priority(&mut self, priority: u8) {
                self.priority.set_bits(0..5, priority);
            }
            fn get_priority(&self) -> u8 {
                self.priority.get_bits(0..5)
            }
        }
        
        impl uavcan::AnonymousFrameHeader for $t {
            const TYPE_ID: u8 = $n;
            
            fn new(priority: u8, discriminator: u16) -> Self {
                Self{
                    priority: priority,
                    discriminator: discriminator,                    
                }
            }
        }
    );
}



#[macro_export]
macro_rules! service_frame_header{
    ($t:ident, $n:expr) => (
        #[derive(Debug, PartialEq)]
        struct $t {
            priority: u8,
            request_not_response: bool,
            destination_node: u8,
            source_node: u8,
        }

        impl uavcan::Header for $t {
            fn id(&self) -> uavcan::transfer::TransferFrameID {
                let mut id = 0;
                id.set_bits(0..7, self.source_node as u32);
                id.set_bit(7, false);
                id.set_bits(8..24, <Self as uavcan::ServiceFrameHeader>::TYPE_ID as u32);
                id.set_bits(24..29, self.priority as u32);
                uavcan::transfer::TransferFrameID::new(id)
            }
            
            fn from_id(id: uavcan::transfer::TransferFrameID) -> Result<Self, ()> {
                if u32::from(id).get_bits(8..24) != <Self as uavcan::ServiceFrameHeader>::TYPE_ID as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: u32::from(id).get_bits(24..29) as u8,
                        request_not_response: u32::from(id).get_bit(15),
                        destination_node: u32::from(id).get_bits(8..14) as u8,
                        source_node: u32::from(id).get_bits(0..7) as u8,
                    })
                }
            }
            fn set_priority(&mut self, priority: u8) {
                self.priority.set_bits(0..5, priority);
            }
            fn get_priority(&self) -> u8 {
                self.priority.get_bits(0..5)
            }
        }
                
        impl uavcan::ServiceFrameHeader for $t {
            const TYPE_ID: u8 = $n;
            
            fn new(priority: u8, request_not_response: bool, source_node: u8, destination_node: u8) -> Self {
                Self{
                    priority: priority,
                    request_not_response: request_not_response,
                    source_node: source_node,   
                    destination_node: destination_node,
                }
            }
        }
    );
}


#[macro_export]
macro_rules! uavcan_frame{
    ($name:ident, $header_type:ident, $body_type:ident, $dts:expr) => (
        struct $name {
            header: $header_type,
            body: $body_type,
        }
        
        uavcan_frame_impls!($name, $header_type, $body_type, $dts);
    );
    ($der:meta, $name:ident, $header_type:ident, $body_type:ident, $dts:expr) => (
        #[$der]
        struct $name {
            header: $header_type,
            body: $body_type,
        }
        uavcan_frame_impls!($name, $header_type, $body_type, $dts);
    );
}

#[macro_export]
macro_rules! uavcan_frame_impls{
    ($name:ident, $header_type:ident, $body_type:ident, $dts:expr) => (
        
        impl uavcan::Frame for $name {
            type Header = $header_type;
            type Body = $body_type;

            const DATA_TYPE_SIGNATURE: u64 = $dts;
            
            fn from_parts(header: $header_type, body: $body_type) -> Self {
                Self{header: header, body: body}
            }
            
            fn to_parts(self) -> ($header_type, $body_type) {
                (self.header, self.body)
            }
            
            fn header(&self) -> & $header_type {
                &self.header
            }
            
            fn body(&self) -> & $body_type {
                &self.body
            }
        }
        
    );
}


#[cfg(test)]
mod tests {

    use types::*;

    use uavcan;

    use {
        DynamicArray,
        PrimitiveType,
        Frame,
    };

    use bit_field::BitField;
    
    #[test]
    fn test_message_frame() {
        
        #[derive(UavcanStruct)]
        struct LogLevel {
            value: u3,
        }

        #[derive(UavcanStruct)]
        struct LogMessage {
            level: LogLevel,
            source: DynamicArray31<u8>,
            text: DynamicArray90<u8>,
        }
        
        message_frame_header!(LogMessageHeader, 16383);
        uavcan_frame!(LogMessageMessage, LogMessageHeader, LogMessage, 0xd654a48e0c049d75);
        
        assert_eq!(LogMessageMessage::DATA_TYPE_SIGNATURE, 0xd654a48e0c049d75);
    }

    #[test]
    fn test_anon_frame() {
        anonymous_frame_header!(GetNodeInfoHeader, 1);
    }


    #[test]
    fn test_service_frame() {
        service_frame_header!(ReadHeader, 48);
    }
}
