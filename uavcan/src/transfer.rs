/// The TransportFrame is uavcan cores main interface to the outside world
///
/// This will in >99% of situations be a CAN2.0B frame
/// But in theory both CAN-FD and other protocols which gives
/// similar guarantees as CAN can also be used
pub trait TransferFrame {
    fn tail_byte(&self) -> TailByte {
        TailByte::from(*self.data().last().unwrap())
    }
    fn is_start_frame(&self) -> bool {
        self.tail_byte().start_of_transfer
    }
    fn is_end_frame(&self) -> bool {
        self.tail_byte().end_of_transfer
    }
    fn is_single_frame(&self) -> bool {
        self.is_end_frame() && self.is_start_frame()
    }

    /// with_data(id: u32, data: &[u]) -> TransportFrame creates a TransportFrame
    /// with an 28 bits ID and data between 0 and the return value ofget_max_data_length()
    fn with_data(id: u32,  data: &[u8]) -> Self;
    fn with_length(id: u32, length: usize) -> Self;
    fn set_data_length(&mut self, length: usize);
    fn max_data_length() -> usize;
    fn data(&self) -> &[u8];
    fn data_as_mut(&mut self) -> &mut[u8];
    fn id(&self) -> u32;
}

pub struct TailByte {
    start_of_transfer: bool,
    end_of_transfer: bool,
    toggle: bool,
    transfer_id: u8,
}

impl From<TailByte> for u8 {
    fn from(tb: TailByte) -> u8 {
        ((tb.start_of_transfer as u8) << 7) | ((tb.end_of_transfer as u8) << 6) | ((tb.toggle as u8) << 5) | (tb.transfer_id&0x1f)
    }
}

impl From<u8> for TailByte {
    fn from(u: u8) -> TailByte {
        TailByte{start_of_transfer: (u&(1<<7)) != 0, end_of_transfer: (u&(1<<6)) != 0, toggle: (u&(1<<5)) != 0, transfer_id: u&0x1f}
    }
}



