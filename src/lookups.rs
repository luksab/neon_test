pub fn double_array_lookup_u4(array: &[u8]) -> Vec<u8> {
    let doubled_array: Vec<u8> = array
        .iter()
        .flat_map(|&x| {
            let high_nibble = (x >> 4) as usize;
            let low_nibble = (x & 0b0000_1111) as usize;

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

            [LOOKUP[high_nibble], LOOKUP[low_nibble]]
        })
        .collect();
    doubled_array
}

pub fn double_array_lookup_u8(array: &[u8]) -> Vec<u8> {
    let lookup: [u16; 256] = (0..=255u8)
        .map(|x| {
            let mut res = 0u16;
            for j in 0..8 {
                let bit = (x as u16 >> j) & 1;
                res |= bit << (j * 2);
                res |= bit << (j * 2 + 1);
            }
            res
        })
        .collect::<Vec<u16>>()
        .try_into()
        .unwrap();

    array
        .iter()
        .flat_map(|&x| lookup[x as usize].to_be_bytes())
        .collect()
}

pub fn double_array_lookup_u16(array: &[u8]) -> Vec<u8> {
    lazy_static::lazy_static! {
        static ref LOOKUP_U16: [u32; 65_536] = {
            let mut v = Vec::with_capacity(65_536);
            for x in 0u32..=65_535 {
                let mut res = 0u32;
                for j in 0..16 {
                    let bit = (x >> j) & 1;
                    res |= bit << (j * 2);
                    res |= bit << (j * 2 + 1);
                }
                v.push(res);
            }
            v.try_into().unwrap()
        };
    }

    array
        .chunks_exact(2)
        .flat_map(|chunk| {
            (&*LOOKUP_U16)[u16::from_be_bytes([chunk[0], chunk[1]]) as usize].to_be_bytes()
        })
        .collect()
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
#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
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
#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
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
