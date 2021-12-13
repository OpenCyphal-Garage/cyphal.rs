/// Inserts a u8 array in a u32 array.
///
/// The u32 must already be initialized to the correct len.
pub fn insert_u8_array_in_u32_array(u8_array: &[u8], u32_array: &mut [u32]) {
    let u32_final_len = (u8_array.len() + 3) / 4;
    // only iterate over n - 1
    for (index, item) in u32_array.iter_mut().enumerate().take(u32_final_len - 1) {
        let index = index * 4;
        let val = slice_into_u32(&u8_array[index..index + 4]);
        *item = val;
    }
    let remaining = match u8_array.len() % 4 {
        0 => 4,
        m => m,
    };
    u32_array[u32_final_len - 1] = slice_into_u32(&u8_array[u8_array.len() - remaining..]);
}

/// Converts a slice of up to 4 bytes to a u32.
///
/// It always takes the first byte of the slice as the most significant byte.
/// If it's less than 4 bytes, the rest is filled with 0.
///
/// u8\[ 34, 234, 23 \] => u32 = 34 | 234 | 23 | 0
fn slice_into_u32(slice: &[u8]) -> u32 {
    // TODO nicer algorithm
    match slice.len() {
        1 => (slice[0] as u32) << 24,
        2 => (slice[0] as u32) << 24 | (slice[1] as u32) << 16,
        3 => (slice[0] as u32) << 24 | (slice[1] as u32) << 16 | (slice[2] as u32) << 8,
        4 => {
            (slice[0] as u32) << 24
                | (slice[1] as u32) << 16
                | (slice[2] as u32) << 8
                | (slice[3] as u32)
        }
        _ => 0,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slice_4_to_u32() {
        let array = [23, 34, 23, 56];
        let val = slice_into_u32(&array);
        assert_eq!(
            val,
            (array[0] as u32) << 24
                | (array[1] as u32) << 16
                | (array[2] as u32) << 8
                | (array[3] as u32)
        );
    }

    #[test]
    fn test_slice_3_to_u32() {
        let array = [23, 34, 23, 56];
        let val = slice_into_u32(&array[..3]);
        assert_eq!(
            val,
            (array[0] as u32) << 24 | (array[1] as u32) << 16 | (array[2] as u32) << 8
        );
    }

    #[test]
    fn test_slice_2_to_u32() {
        let array = [23, 34, 23, 56];
        let val = slice_into_u32(&array[..2]);
        assert_eq!(val, (array[0] as u32) << 24 | (array[1] as u32) << 16);
    }

    #[test]
    fn test_slice_1_to_u32() {
        let array = [23, 34, 23, 56];
        let val = slice_into_u32(&array[..1]);
        assert_eq!(val, (array[0] as u32) << 24);
    }

    #[test]
    fn test_slice_0_to_u32() {
        let array = [23, 34, 23, 56];
        let val = slice_into_u32(&array[..0]);
        assert_eq!(val, 0);
    }

    #[test]
    fn test_u8_array_into_u32_array_len_multiple_of_4() {
        let u8a = [23, 34, 23, 56];
        let mut u32a = [0u32; 1];
        insert_u8_array_in_u32_array(&u8a, &mut u32a);
        assert_eq!(u32a[0], slice_into_u32(&u8a));
    }

    #[test]
    fn test_u8_array_into_u32_array_len_remaining() {
        let u8a = [23, 34, 23, 56, 54, 76];
        let mut u32a = [0u32; 2];
        insert_u8_array_in_u32_array(&u8a, &mut u32a);
        assert_eq!(u32a[0], slice_into_u32(&u8a[0..4]));
        assert_eq!(u32a[1], slice_into_u32(&u8a[4..]));
    }
}
