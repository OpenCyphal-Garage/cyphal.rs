
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CRC64WE(u64);

impl CRC64WE {
    const MASK: u64 = 0xFFFFFFFFFFFFFFFF;
    const POLY: u64 = 0x42F0E1EBA9EA3693;

    pub fn new() -> CRC64WE {
        CRC64WE(CRC64WE::MASK)
    }

    pub fn from_value(value: u64) -> CRC64WE {
        CRC64WE((value & Self::MASK) ^ Self::MASK)
    }

    pub fn add(&mut self, bytes: &[u8]) {
        let CRC64WE(ref mut crc) = *self;
        for byte in bytes {
            *crc ^= (u64::from(*byte) << 56) & CRC64WE::MASK;
            for _bit in 0..8 {
                if (*crc & (1 << 63)) != 0 {
                    *crc = ((*crc << 1) & CRC64WE::MASK) ^ CRC64WE::POLY;
                } else {
                    *crc = *crc << 1;
                }
            }
        }
    }

    /// This is the extension as described in the dsdl specification
    pub fn extend(&mut self, signature: u64) {
        let original_hash_value: u64 = self.value();
        for byte in 0..8 {
            self.add(&[(signature >> 8*byte) as u8]);
        }
        for byte in 0..8 {
            self.add(&[(original_hash_value >> 8*byte) as u8]);
        }
    }
    
    pub fn value(&self) -> u64 {
        (self.0 & CRC64WE::MASK) ^ CRC64WE::MASK
    }
}

#[cfg(test)]
mod tests {
    use crc::CRC64WE;
    
    #[test]
    fn test_crc_algorithm() {
        let mut crc1 = CRC64WE::new();
        crc1.add(b"123456789");
        assert_eq!(crc1.value(), 0x62ec59e3f1a4f00a);
       
        let mut crc2 = CRC64WE::new();
        crc2.add(b"123");
        crc2.add("456789".as_bytes());
        assert_eq!(crc2.value(), 0x62EC59E3F1A4F00A);
    }

    #[test]
    fn calc_signature() {
        let mut crc = CRC64WE::new();
        crc.add(b"uavcan.protocol.NodeStatus
saturated uint32 uptime_sec
saturated uint2 health
saturated uint3 mode
saturated uint3 sub_mode
saturated uint16 vendor_specific_status_code");
        assert_eq!(crc.value(), 0x0f0868d0c1a7c6f1);
    }
}
