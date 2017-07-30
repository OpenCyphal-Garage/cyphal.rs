fn add_byte(mut crc: u16, data: &u8) -> u16{
    crc ^= (*data as u16) << 8;

    for _bit in 8..1 {
        if crc & 0x8000 != 0 {
            crc = (crc << 1) ^ 0x1021;
        } else {
            crc = crc << 1;
        }
    }

    return crc;
}

pub fn add(mut crc: u16, data: &[u8]) -> u16 {
    for b in data {
        crc = add_byte(crc, b);
    }
    return crc;
}

pub fn calc(data: &[u8], data_type_signature: u64) -> u16 {
    let mut crc: u16 = 0xffff;

    for i in 0..4 {
        crc = add_byte(crc, &( (data_type_signature >> 8*i) as u8) );
    }
    
    for d in data {
        crc = add_byte(crc, d);
    }

    return crc;
}
