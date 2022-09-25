pub fn log2(mut n: u64) -> u8 {
    if n == 0 {
        return 0u8;
    }

    let mut i = 0u8;

    while n != 0 {
        n >>= 1;
        i += 1;
    }

    i - 1
}
