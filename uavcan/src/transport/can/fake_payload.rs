use super::TailByte;
use crate::{crc16::Crc16, types::TransferId};

pub struct FakePayloadIter<const N: usize> {
    frame_amount: usize,
    crc: Option<Crc16>,
    act_frame: usize,
    fake_data_count: u8,
    transfer_id: TransferId,
    payload: [u8; N],
}

impl<const N: usize> FakePayloadIter<N> {
    fn new(frame_amount: usize, transfer_id: TransferId) -> Self {
        Self {
            crc: match frame_amount {
                1 => None,
                _ => Some(Crc16::init()),
            },
            frame_amount: frame_amount,
            act_frame: 0,
            fake_data_count: 0,
            transfer_id,
            payload: [0; N],
        }
    }

    pub fn single_frame(transfer_id: TransferId) -> Self {
        Self::new(1, transfer_id)
    }

    pub fn multi_frame(frame_amount: usize, transfer_id: TransferId) -> Self {
        Self::new(frame_amount, transfer_id)
    }
}

impl<const N: usize> core::iter::Iterator for FakePayloadIter<N> {
    type Item = [u8; N];

    fn next(&mut self) -> Option<Self::Item> {
        let stop_iter = self.act_frame == self.frame_amount;

        if stop_iter {
            return None;
        }

        let start = self.act_frame == 0;
        let end = self.act_frame == self.frame_amount - 1;
        let max_payload = N;
        let tail = TailByte::new(start, end, self.act_frame % 2 == 0, self.transfer_id);

        let single_frame_transfer = self.frame_amount == 1;

        if single_frame_transfer {
            let payload_data_ref = &mut self.fake_data_count;
            self.payload[..7].fill_with(|| {
                let tmp = *payload_data_ref;
                *payload_data_ref += 1;
                tmp
            });
        } else {
            let payload = match end {
                true => &mut self.payload[..max_payload - 3],
                _ => &mut self.payload[..max_payload - 1],
            };
            let payload_data_ref = &mut self.fake_data_count;
            payload.fill_with(|| {
                let tmp = *payload_data_ref;
                *payload_data_ref += 1;
                tmp
            });
            let crc = self.crc.as_mut().unwrap();
            crc.digest(payload);

            if end {
                let crc = crc.get_crc().to_be_bytes();
                self.payload[5] = crc[0];
                self.payload[6] = crc[1];
            }
        }
        self.payload[max_payload - 1] = tail;
        self.act_frame += 1;
        Some(self.payload.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::crc16::Crc16;

    use super::{super::TailByte, FakePayloadIter};

    #[test]
    fn test_single_frame_8_byte() {
        let transfer_id = 0;
        let payload_iter = FakePayloadIter::<8>::single_frame(transfer_id);

        let mut frame_count = 0;
        for payload in payload_iter {
            assert_eq!(payload[7], TailByte::new(true, true, true, transfer_id));
            frame_count += 1;
        }
        assert_eq!(frame_count, 1);
    }

    #[test]
    fn test_multi_frame_2_8_byte() {
        let transfer_id = 0;
        let frame_amount = 2;
        let payload_iter = FakePayloadIter::<8>::multi_frame(frame_amount, transfer_id);

        let mut crc = Crc16::init();
        let mut frame_count = 0;
        for (i, payload) in payload_iter.enumerate() {
            assert_eq!(
                payload[7],
                TailByte::new(i == 0, i == 1, i == 0, transfer_id)
            );
            crc.digest(&payload[..7]);
            frame_count += 1;
        }
        assert_eq!(crc.get_crc(), 0x0000);
        assert_eq!(frame_count, frame_amount);
    }

    #[test]
    fn test_multi_frame_3_8_byte() {
        let transfer_id = 0;
        let frame_amount = 3;
        let payload_iter = FakePayloadIter::<8>::multi_frame(frame_amount, transfer_id);

        let mut crc = Crc16::init();
        let mut frame_count = 0;
        for (i, payload) in payload_iter.enumerate() {
            assert_eq!(
                payload[7],
                TailByte::new(i == 0, i == 2, i % 2 == 0, transfer_id)
            );
            crc.digest(&payload[..7]);
            frame_count += 1;
        }
        assert_eq!(crc.get_crc(), 0x0000);
        assert_eq!(frame_count, frame_amount);
    }
}
