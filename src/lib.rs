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

#[repr(C, align(64))]
struct Array{
    pub data: Vec<u8>,
}

pub fn generate_array(size: usize) -> Vec<u8> {
    // let mut array = vec![0; size];
    let mut array = Array {
        data: vec![0; size],
    }.data;
    // let mut random_start = 25487;

    // initialize rand with a seed
    let mut rng = rand::rngs::StdRng::seed_from_u64(25487);

    for i in 0..size {
        // random_start = random_start * 214013 + 2531011;
        // array[i] = random_start as u8;
        array[i] = rng.gen();
        // array[i] = i as u8;
    }
    array
}

pub fn double_array_sisd_laura_orig(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    let n1 = 0b10001000u8;
    let n2 = 0b01000100u8;
    let n3 = 0b00100010u8;
    let n4 = 0b00010001u8;

    let m1 = 0b0100000001000000u16;
    let m2 = 0b0001000000010000u16;
    let m3 = 0b0000010000000100u16;
    let m4 = 0b0000000100000001u16;

    for i in 0..size {
        let mut a = (((array[i] & n1) as u16).pow(2) & m1) * 3;
        a += (((array[i] & n2) as u16).pow(2) & m2) * 3;
        a += (((array[i] & n3) as u16).pow(2) & m3) * 3;
        a += (((array[i] & n4) as u16).pow(2) & m4) * 3;

        doubled_array[i * 2] += (a >> 8) as u8;
        doubled_array[i * 2 + 1] += a as u8;
    }
    doubled_array
}

pub fn double_array_sisd_laura(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    let n1 = 0b10101000u8;
    let n2 = 0b01000010u8;
    let n3 = 0b00010101u8;

    let m1 = 0b0100010001000000u16;
    let m2 = 0b0001000000000100u16;
    let m3 = 0b0000000100010001u16;

    for i in 0..size {
        let mut a = ((array[i] & n1) as u16).pow(2) & m1;
        a += ((array[i] & n2) as u16).pow(2) & m2;
        a += ((array[i] & n3) as u16).pow(2) & m3;

        a = a * 3;

        doubled_array[i * 2] += (a >> 8) as u8;
        doubled_array[i * 2 + 1] += a as u8;
    }
    doubled_array
}

pub fn double_array_sisd_laura_u32(array: &Vec<u8>) -> Vec<u8> {
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    let n1 = 0b00001001001001000001001001001001u32;
    let n2 = 0b10010010010010010010000000010010u32;
    let n3 = 0b00100100100100000100100100100100u32;
    let n4 = 0b01000000000000101000010010000000u32;

    let m1 = 0b0000000001000001000001000001000000000001000001000001000001000001u64;
    let m2 = 0b0100000100000100000100000100000100000100000000000000000100000100u64;
    let m3 = 0b0000010000010000010000010000000000010000010000010000010000010000u64;
    let m4 = 0b0001000000000000000000000000010001000000000100000100000000000000u64;

    for i in (0..size).step_by(4) {
        // let value = u32::from_le_bytes([array[i], array[i + 1], array[i + 2], array[i + 3]]);
        let value = u32::from_be_bytes(array[i..i + 4].try_into().unwrap());
        let mut a = (((value & n1) as u64).pow(2) & m1) * 3;
        a += (((value & n2) as u64).pow(2) & m2) * 3;
        a += (((value & n3) as u64).pow(2) & m3) * 3;
        a += (((value & n4) as u64).pow(2) & m4) * 3;

        // doubled_array[i * 2] += (a >> 8) as u8;
        // doubled_array[i * 2 + 1] += a as u8;
        // doubled_array[i * 2 + 2] += (a >> 24) as u8;
        // doubled_array[i * 2 + 3] += (a >> 16) as u8;
        // doubled_array[i * 2 + 4] += (a >> 40) as u8;
        // doubled_array[i * 2 + 5] += (a >> 32) as u8;
        // doubled_array[i * 2 + 6] += (a >> 56) as u8;
        // doubled_array[i * 2 + 7] += (a >> 48) as u8;
        doubled_array[i * 2..i * 2 + 8].copy_from_slice(&a.to_be_bytes());
    }
    doubled_array
}

#[cfg(all(
    any(target_arch = "x86_64"),
    target_feature = "avx512f",
    target_feature = "avx512vl",
    target_feature = "avx512bw",
    target_feature = "avx512vbmi",
    target_feature = "avx512vbmi2",
    target_feature = "avx512bitalg"
))]
pub fn double_array_simd_laura(array: &Vec<u8>) -> Vec<u8> {
    use std::arch::x86_64::*;
    let size = array.len();
    let mut doubled_array = vec![0; size * 2];

    let n1 = 0b10101000u8;
    let n2 = 0b01000010u8;
    let n3 = 0b00010101u8;

    let m1 = 0b0100010001000000u16;
    let m2 = 0b0001000000000100u16;
    let m3 = 0b0000000100010001u16;

    let (pre, array, post) = unsafe { array.align_to::<__m256i>() };

    for i in 0..pre.len() {
        let mut a = (((pre[i] & n1) as u16).pow(2) & m1) * 3;
        a += (((pre[i] & n2) as u16).pow(2) & m2) * 3;
        a += (((pre[i] & n3) as u16).pow(2) & m3) * 3;

        doubled_array[i * 2] += (a >> 8) as u8;
        doubled_array[i * 2 + 1] += a as u8;
    }

    unsafe {
        let avx_n1 = _mm512_set1_epi16(n1 as i16);
        let avx_n2 = _mm512_set1_epi16(n2 as i16);
        let avx_n3 = _mm512_set1_epi16(n3 as i16);

        let avx_m1 = _mm512_set1_epi16(m1 as i16);
        let avx_m2 = _mm512_set1_epi16(m2 as i16);
        let avx_m3 = _mm512_set1_epi16(m3 as i16);

        // let avx_three = _mm512_set1_epi16(3);
        let avx_zero = _mm512_set1_epi16(0);

        for i in 0..array.len() {
            let input = _mm512_cvtepu8_epi16(array[i]);

            let mut a = _mm512_and_si512(input, avx_n1);
            a = _mm512_mullo_epi16(a, a);
            a = _mm512_and_si512(a, avx_m1);

            let mut b = _mm512_and_si512(input, avx_n2);
            b = _mm512_mullo_epi16(b, b);
            b = _mm512_and_si512(b, avx_m2);

            let mut c = _mm512_and_si512(input, avx_n3);
            c = _mm512_mullo_epi16(c, c);
            c = _mm512_and_si512(c, avx_m3);

            // This could also be "or"
            // let out = _mm512_add_epi16(_mm512_add_epi16(a, b), c);
            let out = _mm512_or_si512(_mm512_or_si512(a, b), c);
            let out = _mm512_or_si512(out, _mm512_shldi_epi64::<1>(out, avx_zero));

            let swapped = {
                // swap adjacent bytes within each 16-bit lane:
                // e.g. [b1 b0 b3 b2 ...] -> [b0 b1 b2 b3 ...]
                let left = _mm512_slli_epi16::<8>(out);
                let right = _mm512_srli_epi16::<8>(out);
                _mm512_or_si512(left, right)
            };

            _mm512_storeu_si512(
                doubled_array.as_mut_ptr().add(i * 64 + pre.len() * 2) as *mut __m512i,
                swapped,
            );
        }
    }

    for i in 0..post.len() {
        let mut a = (((post[i] & n1) as u16).pow(2) & m1) * 3;
        a += (((post[i] & n2) as u16).pow(2) & m2) * 3;
        a += (((post[i] & n3) as u16).pow(2) & m3) * 3;

        doubled_array[(i + pre.len() + array.len() * 32) * 2] += (a >> 8) as u8;
        doubled_array[(i + pre.len() + array.len() * 32) * 2 + 1] += a as u8;
    }
    doubled_array
}

// pub fn double_array_sisd_laura(array: &Vec<u8>) -> Vec<u8> {
//     let size = array.len();
//     let mut doubled_array = vec![0; size * 2];

//     let n1 = 0b11001100u8;
//     let n2 = 0b00110011u8;

//     let m1 = 0b0101000001010000u16;
//     let m2 = 0b0000010100000101u16;

//     for i in 0..size {
//         let mut a = (((array[i] & n1) as u16).pow(2) & m1) * 3;
//         a += (((array[i] & n2) as u16).pow(2) & m2) * 3;

//         doubled_array[i * 2] += a as u8;
//         doubled_array[i * 2 + 1] += (a >> 8) as u8;
//     }
//     doubled_array
// }

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

#[repr(C, align(16))]
struct LookupTable {
    table: [u8; 16],
}

#[rustfmt::skip]
const LOOKUP: LookupTable = LookupTable {
    table: [
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
    ],
};

#[cfg(all(
    any(target_arch = "x86_64"),
    target_feature = "avx512f",
    target_feature = "avx512vl",
    target_feature = "avx512bw",
    target_feature = "avx512vbmi",
    target_feature = "avx512vbmi2",
    target_feature = "avx512bitalg"
))]
pub fn double_array_lookup_avx_u4(array: &[u8]) -> Vec<u8> {
    use std::arch::x86_64::*;

    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);

    let (pre, array, rest) = unsafe { array.align_to::<__m128i>() };

    for i in pre {
        doubled_array.push(LOOKUP.table[(i >> 4) as usize]);
        doubled_array.push(LOOKUP.table[(i & 0b1111) as usize]);
    }

    unsafe {
        // store LUT in a vector
        let lookup = _mm_load_si128(LOOKUP.table.as_ptr() as *const __m128i);

        let mut_ptr = doubled_array.as_mut_ptr();
        let pre_len_x2 = pre.len() * 2;
        for i in 0..array.len() {
            // let input = vld1q_u8(array.as_ptr().add(i));
            let input = *array.get_unchecked(i);
            // get low half of each byte by masking out the high half

            // mask for low nibble
            let mask = _mm_set1_epi8(0x0f as i8);
            // isolate low nibble
            let input_lo = _mm_and_si128(input, mask);
            // get high nibble by shifting right 4 bits (use 16-bit shift to avoid cross-byte shifts), then mask
            let input_hi = _mm_and_si128(_mm_srli_epi16(input, 4), mask);
            // lookup doubled bytes for low/high nibbles using byte shuffle
            let output_lo = _mm_shuffle_epi8(lookup, input_lo);
            let output_hi = _mm_shuffle_epi8(lookup, input_hi);
            // interleave hi/lo bytes per input byte: [hi0, lo0, hi1, lo1, ...] into two 16-byte vectors
            let out0 = _mm_unpacklo_epi8(output_hi, output_lo);
            let out1 = _mm_unpackhi_epi8(output_hi, output_lo);
            // store the two 16-byte vectors (total 32 bytes)
            _mm_storeu_si128(mut_ptr.add(i * 32 + pre_len_x2) as *mut __m128i, out0);
            _mm_storeu_si128(mut_ptr.add(i * 32 + pre_len_x2 + 16) as *mut __m128i, out1);
        }
        _mm_sfence();
        doubled_array.set_len(array.len() * 32 + pre.len() * 2);
    }

    // deal with the rest of the array
    for i in rest {
        doubled_array.push(LOOKUP.table[(i >> 4) as usize]);
        doubled_array.push(LOOKUP.table[(i & 0b1111) as usize]);
    }

    doubled_array
}

#[cfg(all(
    any(target_arch = "x86_64"),
    target_feature = "avx512f",
    target_feature = "avx512vl",
    target_feature = "avx512bw",
    target_feature = "avx512vbmi",
    target_feature = "avx512vbmi2",
    target_feature = "avx512bitalg"
))]
pub fn double_array_lookup_avx512_u4(array: &[u8]) -> Vec<u8> {
    use std::arch::x86_64::*;

    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);

    let (pre, array, rest) = unsafe { array.align_to::<__m512i>() };

    // println!(
    //     "pre len: {}, array len: {}, rest len: {}",
    //     pre.len(),
    //     array.len(),
    //     rest.len()
    // );

    for i in pre {
        doubled_array.push(LOOKUP.table[(i >> 4) as usize]);
        doubled_array.push(LOOKUP.table[(i & 0b1111) as usize]);
    }

    
    unsafe {
        // store LUT in a vector
        let lookup = _mm_load_si128(LOOKUP.table.as_ptr() as *const __m128i);
        let lookup = _mm512_broadcast_i32x4(lookup);
        // let lookup = _mm512_add_epi(lookup);
        // mask for low nibble
        let mask = _mm512_set1_epi8(0x0f as i8);

        let mut_ptr = doubled_array.as_mut_ptr();
        let pre_len_x2 = pre.len() * 2;
        for i in 0..array.len() {
            // let input = vld1q_u8(array.as_ptr().add(i));
            let input = *array.get_unchecked(i);
            // get low half of each byte by masking out the high half

            // isolate low nibble
            let input_lo = _mm512_and_si512(input, mask);
            // get high nibble by shifting right 4 bits (use 16-bit shift to avoid cross-byte shifts), then mask
            let input_hi = _mm512_and_si512(_mm512_srli_epi16(input, 4), mask);
            // lookup doubled bytes for low/high nibbles using byte shuffle
            let output_lo = _mm512_shuffle_epi8(lookup, input_lo);
            let output_hi = _mm512_shuffle_epi8(lookup, input_hi);
            // interleave hi/lo bytes per input byte: [hi0, lo0, hi1, lo1, ...] into two 16-byte vectors
            let out0 = _mm512_unpacklo_epi8(output_hi, output_lo);
            let out1 = _mm512_unpackhi_epi8(output_hi, output_lo);

            // // swap 128 bit lanes within each 512-bit vector
            // let idx = _mm512_setr_epi64(0, 1, 4, 5, 2, 3, 6, 7);
            // let out0 = _mm512_permutexvar_epi64(
            //     idx,
            //     out0,
            // );
            // let out1 = _mm512_permutexvar_epi64(
            //     idx,
            //     out1,
            // );

            // store location: location in 128 bit lanes of out0_out1
            // 0: 0
            // 1: 4
            // 2: 1
            // 3: 5
            // 4: 2
            // 5: 6
            // 6: 3
            // 7: 7

            // store the two 64-byte vectors (total 128 bytes)
            // _mm512_stream_si512(mut_ptr.add(pre_len_x2 + i * 128) as *mut __m512i, out0);
            // _mm512_stream_si512(mut_ptr.add(pre_len_x2 + i * 128 + 64) as *mut __m512i, out1);
            _mm512_storeu_si512(mut_ptr.add(i * 128 + pre_len_x2) as *mut __m512i, out0);
            _mm512_storeu_si512(mut_ptr.add(i * 128 + pre_len_x2 + 64) as *mut __m512i, out1);
        }
        _mm_sfence();
        doubled_array.set_len(array.len() * 128 + pre.len() * 2);
    }

    // deal with the rest of the array
    for i in rest {
        doubled_array.push(LOOKUP.table[(i >> 4) as usize]);
        doubled_array.push(LOOKUP.table[(i & 0b1111) as usize]);
    }

    doubled_array
}

#[cfg(all(
    any(target_arch = "x86_64"),
    target_feature = "avx512f",
    target_feature = "avx512vl",
    target_feature = "avx512bw",
    target_feature = "avx512vbmi",
    target_feature = "avx512vbmi2",
    target_feature = "avx512bitalg"
))]
pub fn throughput_test(array: &[u8]) -> Vec<u8> {
    use std::arch::x86_64::*;
    // let size = array.len();
    let mut doubled_array: Vec<u8> = Vec::with_capacity(array.len() * 2);

    let (pre, array, rest) = unsafe { array.align_to::<__m256i>() };

    for i in pre {
        doubled_array.push(*i);
        doubled_array.push(*i);
    }

    for (i, &item) in array.iter().enumerate() {
        unsafe {
            let input = _mm512_castsi256_si512(item);
            let output = _mm512_unpacklo_epi8(input, input);

            _mm512_storeu_si512(
                doubled_array.as_mut_ptr().add(i * 64 + pre.len() * 2) as *mut __m512i,
                output,
            );
        }
    }
    unsafe { doubled_array.set_len(array.len() * 64 + pre.len() * 2) };

    for i in rest {
        doubled_array.push(*i);
        doubled_array.push(*i);
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
