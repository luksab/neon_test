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
    // thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Crossplatform(
    //     3.try_into().unwrap(),
    // ))
    // .unwrap();
    let x = 128;
    let array = generate_array(x);
    println!("Original array: ");
    print_array_spaced(&array);
    let rotated_array_sisd = double_array_sisd(&array);
    let rotated_array_sisd_opt = double_array_sisd_opt(&array);
    let rotated_array_sisd_iter = double_array_sisd_opt_iter(&array);
    // let rotated_array_sisd_iter_rayon = double_array_sisd_opt_rayon(&array);
    let rotated_array_lut_simd = double_array_lookup_neon_u4(&array);
    let rotated_array_ben = double_array_ben(&array);
    let rotated_array_benk = double_array_benk(&array);
    let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap();
    let rotated_array_lut_simd_multi = double_array_lookup_neon_u4_multithread(&array, &thread_pool);
    println!("Rotated array: ");
    print_array(&rotated_array_sisd);
    assert_eq!(rotated_array_sisd, rotated_array_sisd_opt);
    assert_eq!(rotated_array_sisd, rotated_array_sisd_iter);
    // assert_eq!(rotated_array_sisd, rotated_array_sisd_iter_rayon);
    assert_eq!(rotated_array_sisd, rotated_array_lut_simd);
    assert_eq!(rotated_array_sisd, rotated_array_lut_simd_multi);
    assert_eq!(rotated_array_sisd, rotated_array_ben);
    assert_eq!(rotated_array_sisd, rotated_array_benk);
}
