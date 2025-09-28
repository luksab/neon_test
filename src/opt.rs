use rayon::prelude::{ParallelBridge, ParallelIterator};

pub fn double_array_sisd_opt(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    // let mut doubled_array = vec![0; size * 2];
    let mut doubled_array = Box::new_uninit_slice(size * 2);

    for i in 0..size {
        let num: u16 = array[i] as u16;
        let num = num & 0b0000_0000_1111_1111 | (num & 0b1111_1111_0000_0000) << 8;
        let num = num & 0b0000_1111_0000_1111 | (num & 0b1111_0000_1111_0000) << 4;
        let num = num & 0b0011_0011_0011_0011 | (num & 0b1100_1100_1100_1100) << 2;
        let num = num & 0b0101_0101_0101_0101 | (num & 0b1010_1010_1010_1010) << 1;
        let num = num | num << 1;
        // doubled_array[i * 2 + 1] = (num & 0b0000_0000_1111_1111) as u8;
        // doubled_array[i * 2] = ((num & 0b1111_1111_0000_0000) >> 8) as u8;
        doubled_array[i * 2 + 1].write((num & 0b0000_0000_1111_1111) as u8);
        doubled_array[i * 2].write(((num & 0b1111_1111_0000_0000) >> 8) as u8);
    }
    // SAFETY: we just wrote to every element of the array
    let array = unsafe { doubled_array.assume_init() };
    Vec::from(array)
}

pub fn double_array_sisd_opt_iter(array: &[u8]) -> Vec<u8> {
    array
        .iter()
        .flat_map(|&x| {
            let num: u16 = x as u16;
            let num = num & 0b0000_1111_0000_1111 | (num & 0b1111_0000_1111_0000) << 4;
            let num = num & 0b0011_0011_0011_0011 | (num & 0b1100_1100_1100_1100) << 2;
            let num = num & 0b0101_0101_0101_0101 | (num & 0b1010_1010_1010_1010) << 1;
            let num = num | num << 1;
            [
                ((num & 0b1111_1111_0000_0000) >> 8) as u8,
                (num & 0b0000_0000_1111_1111) as u8,
            ]
        })
        .collect()
}

pub fn double_array_sisd_opt_rayon(array: &[u8]) -> Vec<u8> {
    let num_chunks = 8;
    let chunk_len = array.len() / num_chunks;
    let size = array.len();
    // let array = &array[..size];
    // let mut doubled_array = vec![0; size * 2];
    let doubled_array = std::cell::SyncUnsafeCell::new(Box::new_uninit_slice(size * 2));

    (0..num_chunks)
        .map(|i| i * chunk_len)
        .par_bridge()
        .for_each(|start| {
            let end = usize::min(start + chunk_len, size);
            for i in start..end {
                let num: u16 = array[i] as u16;
                let num = num & 0b0000_0000_1111_1111 | (num & 0b1111_1111_0000_0000) << 8;
                let num = num & 0b0000_1111_0000_1111 | (num & 0b1111_0000_1111_0000) << 4;
                let num = num & 0b0011_0011_0011_0011 | (num & 0b1100_1100_1100_1100) << 2;
                let num = num & 0b0101_0101_0101_0101 | (num & 0b1010_1010_1010_1010) << 1;
                let num = num | num << 1;
                // doubled_array[i * 2 + 1] = (num & 0b0000_0000_1111_1111) as u8;
                // doubled_array[i * 2] = ((num & 0b1111_1111_0000_0000) >> 8) as u8;
                let doubled_array = unsafe { &mut *doubled_array.get() };
                doubled_array[i * 2 + 1].write((num & 0b0000_0000_1111_1111) as u8);
                doubled_array[i * 2].write(((num & 0b1111_1111_0000_0000) >> 8) as u8);
            }
        });
    // SAFETY: we just wrote to every element of the array
    let array = unsafe { doubled_array.into_inner().assume_init() };
    Vec::from(array)
}

pub fn double_array_sisd_opt_64(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    for i in (0..size).step_by(4) {
        let num: u64 = array[i + 3] as u64
            | (array[i + 2] as u64) << 8
            | (array[i + 1] as u64) << 16
            | (array[i] as u64) << 24;
        // let num: u64 = u64::from_be_bytes([
        //     0,
        //     0,
        //     0,
        //     0,
        //     array[i],
        //     array[i + 1],
        //     array[i + 2],
        //     array[i + 3],
        // ]);
        let num = num
            & 0b0000_0000_0000_0000_1111_1111_1111_1111_0000_0000_0000_0000_1111_1111_1111_1111
            | (num
                & 0b1111_1111_1111_1111_0000_0000_0000_0000_1111_1111_1111_1111_0000_0000_0000_0000)
                << 16;
        let num = num
            & 0b0000_0000_1111_1111_0000_0000_1111_1111_0000_0000_1111_1111_0000_0000_1111_1111
            | (num
                & 0b1111_1111_0000_0000_1111_1111_0000_0000_1111_1111_0000_0000_1111_1111_0000_0000)
                << 8;
        let num = num
            & 0b0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111
            | (num
                & 0b1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000_1111_0000)
                << 4;
        let num = num
            & 0b0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011_0011
            | (num
                & 0b1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100_1100)
                << 2;
        let num = num
            & 0b0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101_0101
            | (num
                & 0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010)
                << 1;
        let num = num | num << 1;
        let num_array = num.to_be_bytes();
        doubled_array[i * 2..i * 2 + 8].copy_from_slice(&num_array);
        // doubled_array[i * 2 + 7] = (num & 0b1111_1111) as u8;
        // doubled_array[i * 2 + 6] = ((num & 0b1111_1111_0000_0000) >> 8) as u8;
        // doubled_array[i * 2 + 5] = ((num & 0b1111_1111_0000_0000_0000_0000) >> 16) as u8;
        // doubled_array[i * 2 + 4] = ((num & 0b1111_1111_0000_0000_0000_0000_0000_0000) >> 24) as u8;
        // doubled_array[i * 2 + 3] =
        //     ((num & 0b1111_1111_0000_0000_0000_0000_0000_0000_0000_0000) >> 32) as u8;
        // doubled_array[i * 2 + 2] =
        //     ((num & 0b1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000) >> 40) as u8;
        // doubled_array[i * 2 + 1] = ((num
        //     & 0b1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000)
        //     >> 48) as u8;
        // doubled_array[i * 2] = ((num
        //     & 0b1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000)
        //     >> 56) as u8;
    }
    doubled_array
}
