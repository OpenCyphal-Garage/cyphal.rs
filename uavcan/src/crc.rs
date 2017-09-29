pub struct TransferCRC(u16);

impl TransferCRC {
    pub fn from_signature(data_type_signature: u64) -> TransferCRC {
        let mut crc = TransferCRC(0xffff);
        for i in 0..8 {
            crc.add_byte(&((data_type_signature >> 8*i) as u8));
        }
        crc
    }
    
    fn add_byte(&mut self, data: &u8) {
        match self {
            &mut TransferCRC(ref mut value) => {
        
                *value ^= (*data as u16) << 8;
                
                for _bit in 0..8 {
                    if (*value & 0x8000) != 0 {
                        *value = (*value << 1) ^ 0x1021;
                    } else {
                        *value = *value << 1;
                    }
                }
            },
        }
    }
    
    pub fn add(&mut self, data: &[u8]) {
        for b in data {
            self.add_byte(b);
        }
    }
}

impl From<TransferCRC> for u16 {
    fn from(crc: TransferCRC) -> Self {
        let TransferCRC(value) = crc;
        value
    }
}

#[cfg(test)]
mod tests {

    use crc::TransferCRC;
    
    #[test]
    fn test_add_byte() {
        let mut crc = TransferCRC(0xffff);
        crc.add_byte(&1);
        assert_eq!(u16::from(crc), 0xf1d1);
    }

    #[test]
    fn test_add_bytes() {
        let mut crc = TransferCRC(0xffff);
        crc.add(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(u16::from(crc), 0x3b0a);
    }

    #[test]
    fn test_from_signature() {
        let crc = TransferCRC::from_signature(0xd654a48e0c049d75);
        assert_eq!(u16::from(crc), 0x4570);
    }
}
