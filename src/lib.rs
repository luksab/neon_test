#![feature(stdsimd)]
#![feature(new_uninit)]

pub fn generate_array(size: usize) -> Vec<u8> {
    let mut array = vec![0; size];
    let mut random_start = 25487;

    for i in 0..size {
        random_start = random_start * 214013 + 2531011;
        array[i] = random_start as u8;
    }
    array
}

/// doube up each bit in the array
/// [1|2|3|4|5|6|7|8, 9|10|11|12|13|14|15|16] ->
/// [1|1|2|2|3|3|4|4, 5|5|6|6|7|7|8|8, 9|9|10|10|11|11|12|12, 13|13|14|14|15|15|16|16]
pub fn double_array_sisd(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    for i in 0..size {
        for j in 0..8 {
            let byte = 1 - (j / 4);
            let bit = (array[i] >> j) & 1;
            doubled_array[i * 2 + byte] |= bit << ((j * 2) % 8);
            doubled_array[i * 2 + byte] |= bit << ((j * 2 + 1) % 8);
        }
    }
    doubled_array
}

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
            let num = num & 0b0000_0000_1111_1111 | (num & 0b1111_1111_0000_0000) << 8;
            let num = num & 0b0000_1111_0000_1111 | (num & 0b1111_0000_1111_0000) << 4;
            let num = num & 0b0011_0011_0011_0011 | (num & 0b1100_1100_1100_1100) << 2;
            let num = num & 0b0101_0101_0101_0101 | (num & 0b1010_1010_1010_1010) << 1;
            let num = num | num << 1;
            [
                (num & 0b0000_0000_1111_1111) as u8,
                ((num & 0b1111_1111_0000_0000) >> 8) as u8,
            ]
        })
        .collect()
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

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn double_array_simd(array: &Vec<u8>) -> Vec<u8> {
    use std::arch::aarch64::*;
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    for i in (0..size).step_by(16) {
        unsafe {
            // load
            let input = vld1q_u8(array.as_ptr().add(i));

            // copy input into two vectors
            // [1|2|3|4|5|6|7|8, 9|10|11|12|13|14|15|16] ->
            // [1|1|2|2|3|3|4|4, 5|5|6|6|7|7|8|8]
            // [9|9|10|10|11|11|12|12, 13|13|14|14|15|15|16|16]
            let T0 = vtrn1q_u8(input, input);
            let T1 = vtrn2q_u8(input, input);

            // set

            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2), T0);
            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2 + 16), T1);
        }
    }
    doubled_array
}

pub fn print_array(array: &Vec<u8>) {
    let x = array.len();

    for i in 0..x {
        print!("{:08b}", array[i]);
    }
    println!();
}

pub fn print_array_spaced(array: &Vec<u8>) {
    let x = array.len();

    for i in 0..x {
        for j in 0..8 {
            print!(" {}", ((array[i] << j) & 0b10000000) >> 7);
        }
    }
    println!();
}

pub fn print_2d_slice(array: &[u8], x: usize, y: usize) {
    for i in 0..x {
        for j in 0..y {
            print!("{} ", array[i * y + j]);
        }
        println!();
    }
}
