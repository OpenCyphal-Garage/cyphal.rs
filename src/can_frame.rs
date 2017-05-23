pub enum CanID {
    Extended(u32),
    Normal(u16),
}

pub struct CanFrame {
    pub id: CanID,
    pub dlc: u8,
    pub data: [u8; 8],
}
