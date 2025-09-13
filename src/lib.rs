// #![feature(stdsimd)]
#![feature(sync_unsafe_cell)]
#![feature(portable_simd)]

mod lookups;
pub use lookups::*;
mod multipliers;
pub use multipliers::*;
mod opt;
pub use opt::*;

use rand::{Rng, SeedableRng};

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

// #[cfg(all(
//     any(target_arch = "x86_64"),
//     target_feature = "avx512f",
//     target_feature = "avx512vl",
//     target_feature = "avx512bw",
//     target_feature = "avx512vbmi",
//     target_feature = "avx512vbmi2",
//     target_feature = "avx512bitalg"
// ))]
// pub fn double_array_lookup_avx_u4(array: &[u8]) -> Vec<u8> {
//     use std::arch::x86_64::*;

//     let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);
//     let zero = unsafe { _mm256_setzero_si256() };

//     #[rustfmt::skip]
//     const LOOKUP: [u8; 16] = [
//         0b00000000,
//         0b00000011,
//         0b00001100,
//         0b00001111,
//         0b00110000,
//         0b00110011,
//         0b00111100,
//         0b00111111,
//         0b11000000,
//         0b11000011,
//         0b11001100,
//         0b11001111,
//         0b11110000,
//         0b11110011,
//         0b11111100,
//         0b11111111,
//     ];

//     let (pre, array, rest) = unsafe { array.align_to::<__m128i>() };

//     for i in pre {
//         doubled_array.push(LOOKUP[(i >> 4) as usize]);
//         doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
//     }

//     unsafe {
//         // store LUT in a vector
//         let lookup = _mm_load_si128(LOOKUP.as_ptr() as *const __m128i);

//         let mut_ptr = doubled_array.as_mut_ptr();
//         let pre_len_x2 = pre.len() * 2;
//         for i in 0..array.len() {
//             // let input = vld1q_u8(array.as_ptr().add(i));
//             let input = *array.get_unchecked(i);
//             // get low half of each byte by masking out the high half
//             let input_lo = vbicq_u8(input, vdupq_n_u8(0b1111_0000));
//             // get high half of each byte by shifting right 4 bits
//             let input_hi = vshrq_n_u8(input, 4);
//             // lookup the low and high halves from the LUT to double each bit
//             let output_lo = vqtbl1q_u8(lookup, input_lo);
//             let output_hi = vqtbl1q_u8(lookup, input_hi);
//             // combine the low and high halves back into a single vector
//             let output = vzipq_u8(output_hi, output_lo);
//             vst1q_u8_x2(mut_ptr.add(i * 32 + pre_len_x2), output);
//         }
//         doubled_array.set_len(array.len() * 32 + pre.len() * 2);
//     }

//     // deal with the rest of the array
//     for i in rest {
//         doubled_array.push(LOOKUP[(i >> 4) as usize]);
//         doubled_array.push(LOOKUP[(i & 0b1111) as usize]);
//     }

//     doubled_array
// }

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
