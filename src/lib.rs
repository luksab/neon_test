pub fn generate_array(x: usize, y: usize) -> Vec<Vec<u8>> {
    let mut array = vec![vec![0; y]; x];
    let mut count = 0;
    let mut random_start = 25487;

    for i in 0..x {
        for j in 0..y {
            random_start = random_start * 214013 + 2531011;
            array[i][j] = random_start as u8;
            count += 1;
        }
    }
    array
}

pub fn transpose_array_sisd(array: &Vec<Vec<u8>>) -> Vec<u8> {
    let x = array.len();
    let y = array[0].len();
    let mut tansposed_array = vec![0; x * y];

    for i in 0..x {
        for j in 0..y {
            tansposed_array[j * x + i] = array[i][j];
        }
    }
    tansposed_array
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub fn transpose_array_sisd_prefetch(array: &Vec<Vec<u8>>) -> Vec<u8> {
    let x = array.len();
    let y = array[0].len();
    let mut tansposed_array = vec![0; x * y];
    
    for i in 0..x {
        for j in 0..y {
            tansposed_array[j * x + i] = array[i][j];
        }
    }
    tansposed_array
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "arm"),
    target_feature = "neon"
))]
pub fn transpose_array_simd(array: &mut Vec<Vec<u8>>) -> Vec<u8> {
    use std::arch::aarch64::*;
    let x = array.len();
    let y = array[0].len();
    let mut tansposed_array = vec![0; x * y];

    for i in (0..x).step_by(16) {
        for j in (0..y).step_by(16) {
            unsafe {
                let I0 = vld1q_u8(array[i].as_ptr().add(j));
                let I1 = vld1q_u8(array[i + 1].as_ptr().add(j));
                let I2 = vld1q_u8(array[i + 2].as_ptr().add(j));
                let I3 = vld1q_u8(array[i + 3].as_ptr().add(j));
                let I4 = vld1q_u8(array[i + 4].as_ptr().add(j));
                let I5 = vld1q_u8(array[i + 5].as_ptr().add(j));
                let I6 = vld1q_u8(array[i + 6].as_ptr().add(j));
                let I7 = vld1q_u8(array[i + 7].as_ptr().add(j));
                let I8 = vld1q_u8(array[i + 8].as_ptr().add(j));
                let I9 = vld1q_u8(array[i + 9].as_ptr().add(j));
                let I10 = vld1q_u8(array[i + 10].as_ptr().add(j));
                let I11 = vld1q_u8(array[i + 11].as_ptr().add(j));
                let I12 = vld1q_u8(array[i + 12].as_ptr().add(j));
                let I13 = vld1q_u8(array[i + 13].as_ptr().add(j));
                let I14 = vld1q_u8(array[i + 14].as_ptr().add(j));
                let I15 = vld1q_u8(array[i + 15].as_ptr().add(j));

                let K0 = vzipq_u8(I0, I1);
                let K1 = vzipq_u8(I2, I3);
                let K2 = vzipq_u8(I4, I5);
                let K3 = vzipq_u8(I6, I7);
                let K4 = vzipq_u8(I8, I9);
                let K5 = vzipq_u8(I10, I11);
                let K6 = vzipq_u8(I12, I13);
                let K7 = vzipq_u8(I14, I15);

                let T0 = vcombine_u8(vget_low_u8(K0.0), vget_low_u8(K0.1));
                let T1 = vcombine_u8(vget_low_u8(K1.0), vget_low_u8(K1.1));
                let T2 = vcombine_u8(vget_low_u8(K2.0), vget_low_u8(K2.1));
                let T3 = vcombine_u8(vget_low_u8(K3.0), vget_low_u8(K3.1));
                let T4 = vcombine_u8(vget_low_u8(K4.0), vget_low_u8(K4.1));
                let T5 = vcombine_u8(vget_low_u8(K5.0), vget_low_u8(K5.1));
                let T6 = vcombine_u8(vget_low_u8(K6.0), vget_low_u8(K6.1));
                let T7 = vcombine_u8(vget_low_u8(K7.0), vget_low_u8(K7.1));
                let T8 = vcombine_u8(vget_high_u8(K0.0), vget_high_u8(K0.1));
                let T9 = vcombine_u8(vget_high_u8(K1.0), vget_high_u8(K1.1));
                let T10 = vcombine_u8(vget_high_u8(K2.0), vget_high_u8(K2.1));
                let T11 = vcombine_u8(vget_high_u8(K3.0), vget_high_u8(K3.1));
                let T12 = vcombine_u8(vget_high_u8(K4.0), vget_high_u8(K4.1));
                let T13 = vcombine_u8(vget_high_u8(K5.0), vget_high_u8(K5.1));
                let T14 = vcombine_u8(vget_high_u8(K6.0), vget_high_u8(K6.1));
                let T15 = vcombine_u8(vget_high_u8(K7.0), vget_high_u8(K7.1));

                vst1q_u8(tansposed_array.as_mut_ptr().add(i * y + j), T0);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 1) * y + j), T1);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 2) * y + j), T2);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 3) * y + j), T3);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 4) * y + j), T4);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 5) * y + j), T5);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 6) * y + j), T6);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 7) * y + j), T7);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 8) * y + j), T8);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 9) * y + j), T9);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 10) * y + j), T10);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 11) * y + j), T11);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 12) * y + j), T12);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 13) * y + j), T13);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 14) * y + j), T14);
                vst1q_u8(tansposed_array.as_mut_ptr().add((i + 15) * y + j), T15);
            }
        }
    }
    tansposed_array
}

pub fn print_array(array: &Vec<Vec<u8>>) {
    let x = array.len();
    let y = array[0].len();

    for i in 0..x {
        for j in 0..y {
            print!("{} ", array[i][j]);
        }
        println!();
    }
}

pub fn print_2d_slice(array: &[u8], x: usize, y: usize) {
    for i in 0..x {
        for j in 0..y {
            print!("{} ", array[i * y + j]);
        }
        println!();
    }
}
