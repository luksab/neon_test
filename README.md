# neon_test

These are my first forays into the world of SIMD with Rust. The problem I optimized is decepitively simple:

Take a slice of bytes and double it's size by doubling each bit. 
For example, the slice `[0b00000001, 0b00000010]` would become `[0b00000000, 0b00000011, 0b00000000, 0b00001100]`.

## Implementations

### `double_array_sisd`
As a first step, I needed a reference implementation to compare against.

The algorithm works by iterating over each byte of the input slice, and for each byte, iterating over each bit. Separating the bit iteration from the byte by shifting to the right and anding with 1. And finally shifting the bit to the left by the current bit index into the output slice twice.

### `double_array_sisd_opt`
This algorithm operates on a whole byte at once and uses a kind of "divide and conquer" approach in log(n) of the number of bits.
It works by "recursively" working with smaller and smaller strings of bits. Starting with shifting the upper half of the input left by 8, then taking the upper halves of the upper and lower halves and shifting them left by 4, and so on until the whole byte is shifted left by 1.
Now, each bit is separated by a zero, so now we can take the result and or it with itself shifted right by 1.

