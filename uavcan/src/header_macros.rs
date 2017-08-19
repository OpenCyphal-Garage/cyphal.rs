#[macro_export]
macro_rules! message_frame_header{
    ($t:ident, $n:expr) => (
        #[derive(Debug, PartialEq, Default)]
        struct $t {
            priority: u8,
            source_node: u8,
        }        
        
        impl UavcanHeader for $t {
            fn type_id() -> u16 { $n }
            
            fn id(&self) -> u32 {
                let mut id = 0;
                id.set_bits(0..7, self.source_node as u32);
                id.set_bit(7, false);
                id.set_bits(8..24, Self::type_id() as u32);
                id.set_bits(24..29, self.priority as u32);
                return id;
            }
            
            fn from_id(id: u32) -> Result<Self, ()> {
                if id.get_bits(8..24) != Self::type_id() as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: id.get_bits(24..29) as u8,
                        source_node: id.get_bits(0..7) as u8,
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
        
        impl MessageFrameHeader for $t {
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
            type_id: u8,
        }

        impl UavcanHeader for $t {
            fn type_id() -> u16 { $n }

            fn id(&self) -> u32 {
                let mut id = 0;
                id.set_bits(0..7, 0);
                id.set_bit(7, false);
                id.set_bits(8..10, Self::type_id() as u32);
                id.set_bits(10..24, self.discriminator as u32);
                id.set_bits(24..29, self.priority as u32);
                return id;
            }
   
            fn from_id(id: u32) -> Result<Self, ()> {
                if id.get_bits(8..24) != Self::type_id() as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: id.get_bits(24..29) as u8,
                        type_id: Self::type_id(),
                        discriminator: id.get_bits(10..24) as u16,
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
        
        impl AnonymousFrameHeader for $t {
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

        impl UavcanHeader for $t {
            fn type_id() -> u16 { $n }
            
            fn id(&self) -> u32 {
                let mut id = 0;
                id.set_bits(0..7, self.source_node as u32);
                id.set_bit(7, false);
                id.set_bits(8..24, Self::type_id() as u32);
                id.set_bits(24..29, self.priority as u32);
                return id;
            }
            fn from_id(id: u32) -> Result<Self, ()> {
                if id.get_bits(8..24) != Self::type_id() as u32 {
                    Err(())
                } else {
                    Ok(Self{
                        priority: id.get_bits(24..29) as u8,
                        type_id: Self::type_id(),
                        request_not_response: id.get_bit(15),
                        destination_node: id.get_bits(8..14) as u8,
                        source_node: id.get_bits(0..7) as u8,
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
                
        impl ServiceFrameHeader for $t {
            fn new(priority: u8, request_not_response: bool, source_node: u8, destination_node: u8) -> Self {
                Self{
                    priority: priority,
                    request_not_response: request,
                    destination_node: destination_node,
                    source_node: source_node,                    
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

#[allow(unused_macros)]
macro_rules! uavcan_frame_impls{
    ($name:ident, $header_type:ident, $body_type:ident, $dts:expr) => (
        
        impl UavcanFrame<$header_type, $body_type> for $name {
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

            fn data_type_signature(&self) -> u64 { $dts }
        }
        
    );
}
