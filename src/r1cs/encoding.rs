// VarInt
// - Each octet has MSB set to 1 if there is another octet, 0 otherwise.
// - The 7-bit groups are arranged in little-endian order.

fn usize_to_bits(mut n: usize) -> Vec<u8> {
    let mut res = Vec::new();
    while n > 127 {
        res.push((1 << 7) | (n & 127) as u8);
        n >>= 7;
    }
    res.push((n & 127) as u8);
    res
}

fn bits_to_usize(bits: (Vec<u8>, u8)) -> usize {
    let mut res = 0;
    let mut shift = 0;
    for i in 0..bits.0.len() {
        res += (bits.0[i] as usize) << shift;
        shift += 7;
    }
    res += (bits.1 as usize) << shift;
    res
}

named!(
    vlusize<usize>,
    bits!(do_parse!(
        res: many_till!(
            do_parse!(tag_bits!(u8, 1, 1) >> group: take_bits!(u8, 7) >> (group)),
            do_parse!(tag_bits!(u8, 1, 0) >> group: take_bits!(u8, 7) >> (group))
        ) >> (bits_to_usize(res))
    ))
);

// SignedVarInt
// - Each octet has MSB set to 1 if there is another octet, 0 otherwise.
// - The 7-bit groups are arranged in little-endian order.
// - Integers are encoded by placing the sign bit in the LSB of the first
//   group, and all other bits shifted by 1 to the left.

fn i64_to_bits(n: i64) -> Vec<u8> {
    let mut n: u64 = ((n << 1) ^ (n >> 63)) as u64;
    let mut res = Vec::new();
    while n > 127 {
        res.push((1 << 7) | (n & 127) as u8);
        n >>= 7;
    }
    res.push((n & 127) as u8);
    res
}

fn bits_to_i64(bits: (Vec<u8>, u8)) -> i64 {
    let mut res = 0;
    let mut shift = 0;
    for i in 0..bits.0.len() {
        res += (bits.0[i] as u64) << shift;
        shift += 7;
    }
    res += (bits.1 as u64) << shift;
    if res & 1 == 0 {
        (res >> 1) as i64
    } else {
        -1 * ((res >> 1) + 1) as i64
    }
}

named!(
    vli64<i64>,
    bits!(do_parse!(
        res: many_till!(
            do_parse!(tag_bits!(u8, 1, 1) >> group: take_bits!(u8, 7) >> (group)),
            do_parse!(tag_bits!(u8, 1, 0) >> group: take_bits!(u8, 7) >> (group))
        ) >> (bits_to_i64(res))
    ))
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlusize() {
        macro_rules! eval {
            ($value:expr, $expected:expr) => {
                let res = usize_to_bits($value);
                assert_eq!(&res, $expected);
                match vlusize(&res) {
                    Ok((_, n)) => assert_eq!(n, $value),
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
            };
        }

        eval!(0, &[0]);
        eval!(1, &[1]);
        eval!(2, &[2]);
        eval!(3, &[3]);
        eval!(127, &[127]);
        eval!(128, &[128, 1]);
        eval!(129, &[129, 1]);
        eval!(255, &[255, 1]);
        eval!(256, &[128, 2]);
        eval!(383, &[255, 2]);
        eval!(384, &[128, 3]);
        eval!(16383, &[255, 127]);
        eval!(16384, &[128, 128, 1]);
        eval!(16385, &[129, 128, 1]);
        eval!(65535, &[255, 255, 3]);
        eval!(65536, &[128, 128, 4]);
        eval!(65537, &[129, 128, 4]);
        eval!(2097151, &[255, 255, 127]);
        eval!(2097152, &[128, 128, 128, 1]);
        eval!(2097153, &[129, 128, 128, 1]);
    }

    #[test]
    fn test_vli64() {
        macro_rules! eval {
            ($value:expr, $expected:expr) => {
                let res = i64_to_bits($value);
                assert_eq!(&res, $expected);
                match vli64(&res) {
                    Ok((_, n)) => assert_eq!(n, $value),
                    Err(e) => panic!("Unexpected error: {:?}", e),
                }
            };
        }

        eval!(0, &[0]);
        eval!(-1, &[1]);
        eval!(1, &[2]);
        eval!(-2, &[3]);
        eval!(2, &[4]);
        eval!(-63, &[125]);
        eval!(63, &[126]);
        eval!(-64, &[127]);
        eval!(64, &[128, 1]);
        eval!(-65, &[129, 1]);
        eval!(-128, &[255, 1]);
        eval!(128, &[128, 2]);
        eval!(-192, &[255, 2]);
        eval!(192, &[128, 3]);
        eval!(-8192, &[255, 127]);
        eval!(8192, &[128, 128, 1]);
        eval!(-8193, &[129, 128, 1]);
        eval!(-32768, &[255, 255, 3]);
        eval!(32768, &[128, 128, 4]);
        eval!(-32769, &[129, 128, 4]);
        eval!(-1048576, &[255, 255, 127]);
        eval!(1048576, &[128, 128, 128, 1]);
        eval!(-1048577, &[129, 128, 128, 1]);
    }
}
