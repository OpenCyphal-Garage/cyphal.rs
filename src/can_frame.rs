use {
    TransportFrame,
};

#[derive(Debug, PartialEq)]
pub enum CanID {
    Extended(u32),
    Normal(u16),
}

#[derive(Debug, PartialEq)]
pub struct CanFrame {
    pub id: CanID,
    pub dlc: usize,
    pub data: [u8; 8],
}

impl TransportFrame for CanFrame {
    fn with_data(id: u32, data: &[u8]) -> CanFrame {
        let mut can_data = [0; 8];
        can_data[0..data.len()].clone_from_slice(data);
        CanFrame{id: CanID::Extended(id), dlc: data.len(), data: can_data}
    }

    fn with_length(id: u32, length: usize) -> CanFrame {
        CanFrame{id: CanID::Extended(id), dlc: length, data: [0; 8]}
    }
    
    fn max_data_length() -> usize {
        8
    }

    fn set_data_length(&mut self, length: usize) {
        assert!(length <= 8);
        self.dlc = length;
    }

    fn data(&self) -> &[u8] {
        &self.data[0..self.dlc]
    }

    fn data_as_mut(&mut self) -> &mut[u8] {
        &mut self.data[0..self.dlc]
    }
    
    fn id(&self) -> u32 {
        match self.id {
            CanID::Extended(x) => x,
            CanID::Normal(x) => x as u32,
        }
    }
}
            
