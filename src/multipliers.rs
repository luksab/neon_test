pub fn double_array_benk(array: &[u8]) -> Vec<u8> {
    const L: usize = 6;
    const MASK1: [u64; L] = [290456853, 580913706, 1140936768, 2281873536, 262144, 524288];
    const MASK2: [u64; L] = [
        72357760713425169,
        289431042853700676,
        1157425108814401536,
        4629700435257606144,
        68719476736,
        274877906944,
    ];

    pub fn double(x: u32) -> u64 {
        let mut res = 0;
        for i in 0..L {
            let y = (x as u64) & MASK1[i];
            res |= (y * y) & MASK2[i];
        }

        res | (res << 1)
    }

    let mut doubled_array = vec![0; array.len() * 2];

    for i in (0..array.len()).step_by(4) {
        let num = u32::from_le_bytes([array[i], array[i + 1], array[i + 2], array[i + 3]]);
        let num = double(num);
        let num_array = num.to_le_bytes();
        doubled_array[i * 2..i * 2 + 8].copy_from_slice(&num_array);
    }
    doubled_array
}

pub fn double_array_ben(array: &[u8]) -> Vec<u8> {
    fn double(x: u8) -> u16 {
        let a = (((((x as u64) * 0x0101010101010101u64) & 0x8040201008040201u64)
            * 0x0102040810204081u64)
            >> 49)
            & 0x5555;
        let b = (((((x as u64) * 0x0101010101010101u64) & 0x8040201008040201u64)
            * 0x0102040810204081u64)
            >> 48)
            & 0xAAAA;
        (a | b) as u16
    }

    let mut doubled_array = vec![0; array.len() * 2];

    for i in 0..array.len() {
        let num = double(array[i]);
        doubled_array[i * 2 + 1] = (num & 0b0000_0000_1111_1111) as u8;
        doubled_array[i * 2] = ((num & 0b1111_1111_0000_0000) >> 8) as u8;
    }
    doubled_array
}
