#![feature(stdsimd)]
#![feature(new_uninit)]
#![feature(sync_unsafe_cell)]

use std::mem::MaybeUninit;

use rand::{Rng, SeedableRng};
use rayon::prelude::{ParallelBridge, ParallelIterator};

pub fn generate_array(size: usize) -> Vec<u8> {
    let mut array = vec![0; size];
    // let mut random_start = 25487;

    // initialize rand with a seed
    let mut rng = rand::rngs::StdRng::seed_from_u64(25487);

    for i in 0..size {
        // random_start = random_start * 214013 + 2531011;
        // array[i] = random_start as u8;
        array[i] = rng.gen();
    }
    array
}

/// double up each bit in the array
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

// const NUM_THREADS: usize = 8;
// static mut CHANNELS: Option<
//     [(
//         kanal::Sender<(&[u8], &mut [MaybeUninit<u8>])>,
//         kanal::Receiver<()>,
//     ); NUM_THREADS],
// > = None;
// pub fn double_array_sisd_opt_channel(in_array: &[u8]) -> Vec<u8> {
//     let channels = unsafe {
//         &mut *CHANNELS.get_or_insert_with(|| {
//             let mut channels: Vec<_> = Vec::with_capacity(NUM_THREADS);
//             for i in 0..NUM_THREADS {
//                 let (send_tx, send_rx) = kanal::bounded(1);
//                 let (recv_tx, recv_rx) = kanal::bounded(1);
//                 channels.push((send_tx, recv_rx));
//                 std::thread::spawn(move || {
//                     loop {
//                         let (in_array, out_array): (&[u8], &mut [MaybeUninit<u8>]) =
//                             send_rx.recv().unwrap();
//                         let size = in_array.len();
//                         for i in 0..size {
//                             let num: u16 = in_array[i] as u16;
//                             let num =
//                                 num & 0b0000_0000_1111_1111 | (num & 0b1111_1111_0000_0000) << 8;
//                             let num =
//                                 num & 0b0000_1111_0000_1111 | (num & 0b1111_0000_1111_0000) << 4;
//                             let num =
//                                 num & 0b0011_0011_0011_0011 | (num & 0b1100_1100_1100_1100) << 2;
//                             let num =
//                                 num & 0b0101_0101_0101_0101 | (num & 0b1010_1010_1010_1010) << 1;
//                             let num = num | num << 1;
//                             // doubled_array[i * 2 + 1] = (num & 0b0000_0000_1111_1111) as u8;
//                             // doubled_array[i * 2] = ((num & 0b1111_1111_0000_0000) >> 8) as u8;
//                             out_array[i * 2 + 1].write((num & 0b0000_0000_1111_1111) as u8);
//                             out_array[i * 2].write(((num & 0b1111_1111_0000_0000) >> 8) as u8);
//                         }
//                         recv_tx.send(()).unwrap();
//                     }
//                 });
//             }
//             channels
//                 .into_iter()
//                 .map(|(sender, receiver)| (sender.clone(), receiver.clone()))
//                 .collect::<Vec<_>>()
//                 .try_into()
//                 .unwrap()
//         })
//     };

//     let chunk_len = in_array.len() / NUM_THREADS;
//     let size = in_array.len();
//     // let array = &array[..size];
//     // let mut doubled_array = vec![0; size * 2];
//     let doubled_array = std::cell::SyncUnsafeCell::new(Box::new_uninit_slice(size * 2));

//     let split_in_array = in_array.chunks(chunk_len);
//     let split_doubled_array = unsafe { &mut *doubled_array.get() };
//     for i in 0..NUM_THREADS {
//         let start = i * chunk_len;
//         let end = usize::min(start + chunk_len, size);
//         let (send_tx, recv_rx) = &channels[i];
//         send_tx
//             .send((&in_array[start..end], &mut doubled_array))
//             .unwrap();
//     }
//     // SAFETY: we just wrote to every element of the array
//     let array = unsafe { doubled_array.into_inner().assume_init() };
//     Vec::from(array)
// }

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

pub fn print_array(array: &[u8]) {
    let x = array.len();

    for i in 0..x {
        print!("{:08b}", array[i]);
    }
    println!();
}

pub fn print_array_spaced(array: &[u8]) {
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

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn double_array_lookup_neon_u4(array: &[u8]) -> Vec<u8> {
    use std::arch::aarch64::*;

    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);

    #[rustfmt::skip]
    const LOOKUP: [u8; 16] = [
        0b00000000,
        0b00000011,
        0b00001100,
        0b00001111,
        0b00110000,
        0b00110011,
        0b00111100,
        0b00111111,
        0b11000000,
        0b11000011,
        0b11001100,
        0b11001111,
        0b11110000,
        0b11110011,
        0b11111100,
        0b11111111,
    ];

    let (pre, array, rest) = unsafe { array.align_to::<uint8x16_t>() };

    for i in pre {
        doubled_array.push(LOOKUP[(i >> 4) as usize]);
        doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
    }

    unsafe {
        // store LUT in a vector
        let lookup = vld1q_u8(LOOKUP.as_ptr());

        let mut_ptr = doubled_array.as_mut_ptr();
        let pre_len_x2 = pre.len() * 2;
        for i in 0..array.len() {
            // let input = vld1q_u8(array.as_ptr().add(i));
            let input = *array.get_unchecked(i);
            // get low half of each byte by masking out the high half
            let input_lo = vbicq_u8(input, vdupq_n_u8(0b1111_0000));
            // get high half of each byte by shifting right 4 bits
            let input_hi = vshrq_n_u8(input, 4);
            // lookup the low and high halves from the LUT to double each bit
            let output_lo = vqtbl1q_u8(lookup, input_lo);
            let output_hi = vqtbl1q_u8(lookup, input_hi);
            // combine the low and high halves back into a single vector
            let output = vzipq_u8(output_hi, output_lo);
            vst1q_u8_x2(mut_ptr.add(i * 32 + pre_len_x2), output);
        }
        doubled_array.set_len(array.len() * 32 + pre.len() * 2);
    }

    // deal with the rest of the array
    for i in rest {
        doubled_array.push(LOOKUP[(i >> 4) as usize]);
        doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
    }

    doubled_array
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn double_array_lookup_neon_u4_slice(array: &[u8], doubled_array: &mut [u8]) {
    use std::arch::aarch64::*;

    assert_eq!(array.len() * 2, doubled_array.len());

    #[rustfmt::skip]
    const LOOKUP: [u8; 16] = [
        0b00000000,
        0b00000011,
        0b00001100,
        0b00001111,
        0b00110000,
        0b00110011,
        0b00111100,
        0b00111111,
        0b11000000,
        0b11000011,
        0b11001100,
        0b11001111,
        0b11110000,
        0b11110011,
        0b11111100,
        0b11111111,
    ];

    let (pre, array, rest) = unsafe { array.align_to::<uint8x16_t>() };

    for (i, num) in pre.iter().enumerate() {
        // doubled_array.push(LOOKUP[(i >> 4) as usize]);
        // doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
        doubled_array[i * 2] = LOOKUP[(num >> 4) as usize];
        doubled_array[i * 2 + 1] = LOOKUP[(num & 0b1111) as usize];
    }

    unsafe {
        // store LUT in a vector
        let lookup = vld1q_u8(LOOKUP.as_ptr());

        let mut_ptr = doubled_array.as_mut_ptr();
        let pre_len_x2 = pre.len() * 2;
        for i in 0..array.len() {
            // let input = vld1q_u8(array.as_ptr().add(i));
            let input = *array.get_unchecked(i);
            // get low half of each byte by masking out the high half
            let input_lo = vbicq_u8(input, vdupq_n_u8(0b1111_0000));
            // get high half of each byte by shifting right 4 bits
            let input_hi = vshrq_n_u8(input, 4);
            // lookup the low and high halves from the LUT to double each bit
            let output_lo = vqtbl1q_u8(lookup, input_lo);
            let output_hi = vqtbl1q_u8(lookup, input_hi);
            // combine the low and high halves back into a single vector
            let output = vzipq_u8(output_hi, output_lo);
            vst1q_u8_x2(mut_ptr.add(i * 32 + pre_len_x2), output);
        }
        // doubled_array.set_len(array.len() * 32 + pre.len() * 2);
    }

    // deal with the rest of the array
    for (i, num) in rest.iter().enumerate() {
        // doubled_array.push(LOOKUP[(i >> 4) as usize]);
        // doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
        doubled_array[i * 2 + pre.len() * 2 + array.len() * 2] = LOOKUP[(num >> 4) as usize];
        doubled_array[i * 2 + pre.len() * 2 + array.len() * 2 + 1] =
            LOOKUP[(num & 0b1111) as usize];
    }
}

/// Splits a slice into `n` equal chunks.
/// The last chunk will be shorter if the length of the slice is not divisible by `n`.
fn split_into_chunks<T>(slice: &[T], n: usize) -> Vec<&[T]> {
    let mut chunks = Vec::with_capacity(n);
    // rounded up
    let chunk_size = (slice.len() + n - 1) / n;

    for i in 0..n {
        let start = i * chunk_size;
        let end = usize::min(start + chunk_size, slice.len());
        chunks.push(&slice[start..end]);
    }

    chunks
}

/// Same as `split_into_chunks`, but returns mutable slices.
/// This is safe because the returned slices are disjoint.
fn split_into_chunks_mut<T>(slice: &mut [T], n: usize) -> Vec<&mut [T]> {
    let mut chunks = Vec::with_capacity(n);
    // rounded up, pretend we're dealing with half the size
    let chunk_size = (slice.len() / 2 + n - 1) / n * 2;

    let len = slice.len();
    // println!("len: {}", len);
    for i in 0..n {
        let start = i * chunk_size;
        // println!("start: {}", start);
        let end = usize::min(start + chunk_size, len);
        // println!("end: {}", end);
        unsafe {
            chunks.push(std::slice::from_raw_parts_mut(
                slice.as_mut_ptr().add(start),
                end - start,
            ));
        }
    }

    chunks
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn double_array_lookup_neon_u4_multithread(
    array: &[u8],
    thread_pool: &rayon::ThreadPool,
) -> Vec<u8> {
    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);
    thread_pool.scope(|s| {
        unsafe {
            doubled_array.set_len(array.len() * 2);
        }
        let doubled_chunks = split_into_chunks_mut(&mut doubled_array, 8);
        let array_chunks = split_into_chunks(array, 8);

        for (doubled_chunk, array_chunk) in doubled_chunks.into_iter().zip(array_chunks) {
            // println!("lengths: {}, {}", doubled_chunk.len(), array_chunk.len());
            s.spawn(|_| {
                double_array_lookup_neon_u4_slice(array_chunk, doubled_chunk);
            });
        }
    });
    unsafe {
        doubled_array.set_len(array.len() * 2);
    }

    doubled_array
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn double_array_lookup_neon_u4_unrolled(array: &[u8]) -> Vec<u8> {
    use std::arch::aarch64::*;

    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);

    #[rustfmt::skip]
    const LOOKUP: [u8; 16] = [
        0b00000000,
        0b00000011,
        0b00001100,
        0b00001111,
        0b00110000,
        0b00110011,
        0b00111100,
        0b00111111,
        0b11000000,
        0b11000011,
        0b11001100,
        0b11001111,
        0b11110000,
        0b11110011,
        0b11111100,
        0b11111111,
    ];

    let (array, rest) = array.split_at(array.len() - array.len() % 16);

    unsafe {
        // store LUT in a vector
        let lookup = vld1q_u8(LOOKUP.as_ptr());

        for i in (0..array.len()).step_by(32) {
            let input = vld1q_u8(array.as_ptr().add(i));
            // get low half of each byte by masking out the high half
            let input_lo = vbicq_u8(input, vdupq_n_u8(0b1111_0000));
            // get high half of each byte by shifting right 4 bits
            let input_hi = vshrq_n_u8(input, 4);
            let input2 = vld1q_u8(array.as_ptr().add(i + 16));
            let input_lo2 = vbicq_u8(input2, vdupq_n_u8(0b1111_0000));
            let input_hi2 = vshrq_n_u8(input2, 4);
            // lookup the low and high halves from the LUT to double each bit
            let output_lo = vqtbl1q_u8(lookup, input_lo);
            let output_hi = vqtbl1q_u8(lookup, input_hi);
            let output = vzipq_u8(output_hi, output_lo);
            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2), output.0);
            let output_lo2 = vqtbl1q_u8(lookup, input_lo2);
            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2 + 16), output.1);
            let output_hi2 = vqtbl1q_u8(lookup, input_hi2);
            // combine the low and high halves back into a single vector
            let output2 = vzipq_u8(output_hi2, output_lo2);
            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2 + 32), output2.0);
            vst1q_u8(doubled_array.as_mut_ptr().add(i * 2 + 48), output2.1);
        }
        doubled_array.set_len(array.len() * 2);
    }

    // deal with the rest of the array
    for i in rest {
        doubled_array.push(LOOKUP[(i >> 4) as usize]);
        doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
    }

    doubled_array
}
