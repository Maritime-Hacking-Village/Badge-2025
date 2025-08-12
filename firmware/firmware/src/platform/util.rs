pub fn bool_array_to_u8(arr: &[bool; 8]) -> u8 {
    arr.iter().fold(0, |acc, &b| (acc << 1) | b as u8)
}

// # [false, true, false, true] => 0b0101 = 0x05
