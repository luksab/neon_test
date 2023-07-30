// #[cfg(
//     all(
//         any(target_arch = "aarch64", target_arch = "arm"),
//         target_feature = "neon"
//     )
// )]
// pub fn rotate_array_neon(a: &[u8], b: &mut [u8]) {
//     use std::arch::aarch64::*;

//     let a = a.as_ptr();
//     let b = b.as_mut_ptr();

//     unsafe {
//         let a = vld1q_u8(a);
//         let c = vrev64q_u8(a);
//         vst1q_u8(b, c);
//     }
// }

// transpose an x by y 2d array
// x is the number of rows
// y is the number of columns

use neon_test::*;

fn main() {
    thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Crossplatform(
        3.try_into().unwrap(),
    ))
    .unwrap();
    let x = 16;
    let y = 16;
    let mut array = generate_array(x, y);
    println!("Original array: ");
    print_array(&array);
    let rotated_array = transpose_array_sisd(&array);
    println!("Rotated array: ");
    print_2d_slice(&rotated_array, x, y);
    // simd
    let rotated_array_simd = transpose_array_simd(&mut array);
    println!("Rotated array simd: ");
    print_2d_slice(&rotated_array_simd, x, y);
    assert_eq!(rotated_array, rotated_array_simd)
}
