use std::ops::*;

pub fn bit_at(opcode: u16, pos: u8) -> bool {
    opcode.bitand(1.shl(pos as u16) as u16) != 0
}

pub fn set_bit_at(opcode: u16, pos: u16, v: bool) -> u16 {
    if v {
        opcode.bitor(1u16 << pos)
    } else {
        opcode.bitand(1u16.shl(pos).bitxor(0xFFFF))
    }
}

pub fn sub_sum(e: u16, left: u8, right: u8) -> u16 {
    let mut sum = 0u16;

    for i in (right..=left).rev() {
        sum <<= 1;
        if bit_at(e, i) {
            sum += 1;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use crate::utils::bit_registers::sub_sum;
    #[test]
    fn test_sub_sum() {
        assert_eq!(sub_sum(0xFu16, 3, 0), 0xF);
        assert_eq!(sub_sum(0xFu16, 2, 0), 0x7);
        assert_eq!(sub_sum(0xCu16, 3, 0), 0xC);
        assert_eq!(sub_sum(0xCu16, 2, 0), 0x4);
        assert_eq!(sub_sum(0xAu16, 3, 0), 0xA);
        assert_eq!(sub_sum(0xAu16, 2, 0), 0x2);
    }
}
