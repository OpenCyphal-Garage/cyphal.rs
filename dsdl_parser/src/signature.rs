

struct CRC64WE(u64);

impl CRC64WE {
    const MASK: u64 = 0xFFFFFFFFFFFFFFFF;
    const POLY: u64 = 0x42F0E1EBA9EA3693;

    fn new() -> CRC64WE {
        CRC64WE(CRC64WE::MASK)
    }

    fn add(&mut self, bytes: &[u8]) {
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

    fn value(&self) -> u64 {
        (self.0 & CRC64WE::MASK) ^ CRC64WE::MASK
    }
}

#[cfg(test)]
mod tests {
    use *;

    use signature::CRC64WE;
    
    #[test]
    fn test_crc_algorithm() {
        let mut crc = CRC64WE::new();
        crc.add(b"123456789");
        assert_eq!(crc.value(), 0x62ec59e3f1a4f00a);
    }
}
