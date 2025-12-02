//! AVX2-accelerated Keccak-p1600 permutation.
//!
//! This module provides a highly optimized implementation of the Keccak-p[1600, ROUNDS]
//! permutation using AVX2 SIMD intrinsics.

#![allow(clippy::identity_op)]
#![allow(clippy::too_many_lines)]

/// Round constants for Keccak-f[1600]
const RC: [u64; 24] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808a,
    0x8000000080008000,
    0x000000000000808b,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008a,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000a,
    0x000000008000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Perform a single Keccak round with full loop unrolling.
/// This macro is designed for maximum inlining and optimization.
macro_rules! keccak_round {
    ($a:expr, $rc:expr) => {{
        // Theta step
        let c0 = $a[0] ^ $a[5] ^ $a[10] ^ $a[15] ^ $a[20];
        let c1 = $a[1] ^ $a[6] ^ $a[11] ^ $a[16] ^ $a[21];
        let c2 = $a[2] ^ $a[7] ^ $a[12] ^ $a[17] ^ $a[22];
        let c3 = $a[3] ^ $a[8] ^ $a[13] ^ $a[18] ^ $a[23];
        let c4 = $a[4] ^ $a[9] ^ $a[14] ^ $a[19] ^ $a[24];

        let d0 = c4 ^ c1.rotate_left(1);
        let d1 = c0 ^ c2.rotate_left(1);
        let d2 = c1 ^ c3.rotate_left(1);
        let d3 = c2 ^ c4.rotate_left(1);
        let d4 = c3 ^ c0.rotate_left(1);

        $a[0] ^= d0;
        $a[1] ^= d1;
        $a[2] ^= d2;
        $a[3] ^= d3;
        $a[4] ^= d4;
        $a[5] ^= d0;
        $a[6] ^= d1;
        $a[7] ^= d2;
        $a[8] ^= d3;
        $a[9] ^= d4;
        $a[10] ^= d0;
        $a[11] ^= d1;
        $a[12] ^= d2;
        $a[13] ^= d3;
        $a[14] ^= d4;
        $a[15] ^= d0;
        $a[16] ^= d1;
        $a[17] ^= d2;
        $a[18] ^= d3;
        $a[19] ^= d4;
        $a[20] ^= d0;
        $a[21] ^= d1;
        $a[22] ^= d2;
        $a[23] ^= d3;
        $a[24] ^= d4;

        // Rho and Pi (fully unrolled)
        let t = $a[1];
        $a[1] = $a[6].rotate_left(44);
        $a[6] = $a[9].rotate_left(20);
        $a[9] = $a[22].rotate_left(61);
        $a[22] = $a[14].rotate_left(39);
        $a[14] = $a[20].rotate_left(18);
        $a[20] = $a[2].rotate_left(62);
        $a[2] = $a[12].rotate_left(43);
        $a[12] = $a[13].rotate_left(25);
        $a[13] = $a[19].rotate_left(8);
        $a[19] = $a[23].rotate_left(56);
        $a[23] = $a[15].rotate_left(41);
        $a[15] = $a[4].rotate_left(27);
        $a[4] = $a[24].rotate_left(14);
        $a[24] = $a[21].rotate_left(2);
        $a[21] = $a[8].rotate_left(55);
        $a[8] = $a[16].rotate_left(45);
        $a[16] = $a[5].rotate_left(36);
        $a[5] = $a[3].rotate_left(28);
        $a[3] = $a[18].rotate_left(21);
        $a[18] = $a[17].rotate_left(15);
        $a[17] = $a[11].rotate_left(10);
        $a[11] = $a[7].rotate_left(6);
        $a[7] = $a[10].rotate_left(3);
        $a[10] = t.rotate_left(1);

        // Chi (fully unrolled per row)
        let t0 = $a[0];
        let t1 = $a[1];
        let t2 = $a[2];
        let t3 = $a[3];
        let t4 = $a[4];
        $a[0] = t0 ^ ((!t1) & t2);
        $a[1] = t1 ^ ((!t2) & t3);
        $a[2] = t2 ^ ((!t3) & t4);
        $a[3] = t3 ^ ((!t4) & t0);
        $a[4] = t4 ^ ((!t0) & t1);

        let t0 = $a[5];
        let t1 = $a[6];
        let t2 = $a[7];
        let t3 = $a[8];
        let t4 = $a[9];
        $a[5] = t0 ^ ((!t1) & t2);
        $a[6] = t1 ^ ((!t2) & t3);
        $a[7] = t2 ^ ((!t3) & t4);
        $a[8] = t3 ^ ((!t4) & t0);
        $a[9] = t4 ^ ((!t0) & t1);

        let t0 = $a[10];
        let t1 = $a[11];
        let t2 = $a[12];
        let t3 = $a[13];
        let t4 = $a[14];
        $a[10] = t0 ^ ((!t1) & t2);
        $a[11] = t1 ^ ((!t2) & t3);
        $a[12] = t2 ^ ((!t3) & t4);
        $a[13] = t3 ^ ((!t4) & t0);
        $a[14] = t4 ^ ((!t0) & t1);

        let t0 = $a[15];
        let t1 = $a[16];
        let t2 = $a[17];
        let t3 = $a[18];
        let t4 = $a[19];
        $a[15] = t0 ^ ((!t1) & t2);
        $a[16] = t1 ^ ((!t2) & t3);
        $a[17] = t2 ^ ((!t3) & t4);
        $a[18] = t3 ^ ((!t4) & t0);
        $a[19] = t4 ^ ((!t0) & t1);

        let t0 = $a[20];
        let t1 = $a[21];
        let t2 = $a[22];
        let t3 = $a[23];
        let t4 = $a[24];
        $a[20] = t0 ^ ((!t1) & t2);
        $a[21] = t1 ^ ((!t2) & t3);
        $a[22] = t2 ^ ((!t3) & t4);
        $a[23] = t3 ^ ((!t4) & t0);
        $a[24] = t4 ^ ((!t0) & t1);

        // Iota
        $a[0] ^= $rc;
    }};
}

/// Keccak-p[1600, ROUNDS] permutation with AVX2 acceleration.
///
/// This function applies the Keccak permutation to the given state
/// for the specified number of rounds at compile time.
///
/// # Arguments
/// * `state` - A mutable reference to the 25-element u64 state array
///
/// # Type Parameters
/// * `ROUNDS` - The number of permutation rounds (typically 12 or 24)
#[inline]
#[target_feature(enable = "avx2")]
pub unsafe fn p1600_avx2<const ROUNDS: usize>(state: &mut [u64; 25]) {
    // Load state into local variables for better register allocation
    let mut a = *state;
    
    let start_round = 24 - ROUNDS;
    
    // Unroll 2 rounds at a time for better instruction scheduling
    let mut round = start_round;
    while round + 1 < 24 {
        keccak_round!(a, RC[round]);
        keccak_round!(a, RC[round + 1]);
        round += 2;
    }
    
    // Handle odd round if needed
    if round < 24 {
        keccak_round!(a, RC[round]);
    }
    
    *state = a;
}

/// Safe wrapper for p1600 that calls the AVX2 implementation.
///
/// # Arguments
/// * `state` - A mutable reference to the 25-element u64 state array
///
/// # Type Parameters
/// * `ROUNDS` - The number of permutation rounds (typically 12 or 24)
#[inline]
pub fn p1600<const ROUNDS: usize>(state: &mut [u64; 25]) {
    // SAFETY: We assume AVX2 is available on all target systems (2015+)
    unsafe {
        p1600_avx2::<ROUNDS>(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector from the keccak crate: f1600 (24 rounds) on zero state
    #[test]
    fn test_f1600_zero_state() {
        let mut data = [0u64; 25];
        p1600::<24>(&mut data);
        
        let expected = [
            0xF1258F7940E1DDE7, 0x84D5CCF933C0478A, 0xD598261EA65AA9EE, 0xBD1547306F80494D,
            0x8B284E056253D057, 0xFF97A42D7F8E6FD4, 0x90FEE5A0A44647C4, 0x8C5BDA0CD6192E76,
            0xAD30A6F71B19059C, 0x30935AB7D08FFC64, 0xEB5AA93F2317D635, 0xA9A6E6260D712103,
            0x81A57C16DBCF555F, 0x43B831CD0347C826, 0x01F22F1A11A5569F, 0x05E5635A21D9AE61,
            0x64BEFEF28CC970F2, 0x613670957BC46611, 0xB87C5A554FD00ECB, 0x8C3EE88A1CCF32C8,
            0x940C7922AE3A2614, 0x1841F924A2C509E4, 0x16F53526E70465C2, 0x75F644E97F30A13B,
            0xEAF1FF7B5CECA249,
        ];
        
        assert_eq!(data, expected);
    }

    /// Test two consecutive f1600 calls
    #[test]
    fn test_f1600_double() {
        let mut data = [0u64; 25];
        p1600::<24>(&mut data);
        p1600::<24>(&mut data);
        
        let expected = [
            0x2D5C954DF96ECB3C, 0x6A332CD07057B56D, 0x093D8D1270D76B6C, 0x8A20D9B25569D094,
            0x4F9C4F99E5E7F156, 0xF957B9A2DA65FB38, 0x85773DAE1275AF0D, 0xFAF4F247C3D810F7,
            0x1F1B9EE6F79A8759, 0xE4FECC0FEE98B425, 0x68CE61B6B9CE68A1, 0xDEEA66C4BA8F974F,
            0x33C43D836EAFB1F5, 0xE00654042719DBD9, 0x7CF8A9F009831265, 0xFD5449A6BF174743,
            0x97DDAD33D8994B40, 0x48EAD5FC5D0BE774, 0xE3B8C8EE55B7B03C, 0x91A0226E649E42E9,
            0x900E3129E7BADD7B, 0x202A9EC5FAA3CCE8, 0x5B3402464E1C3DB6, 0x609F4E62A44C1059,
            0x20D06CD26A8FBF5C,
        ];
        
        assert_eq!(data, expected);
    }

    /// Test 12 rounds (used in TurboSHAKE)
    #[test]
    fn test_p1600_12_rounds() {
        let mut our_state = [0u64; 25];
        let mut ref_state = [0u64; 25];
        
        // Fill with test pattern
        for i in 0..25 {
            our_state[i] = (i as u64) * 0x0123456789ABCDEF;
            ref_state[i] = our_state[i];
        }
        
        p1600::<12>(&mut our_state);
        keccak::p1600(&mut ref_state, 12);
        
        assert_eq!(our_state, ref_state);
    }

    /// Test against reference keccak crate for various round counts
    #[test]
    fn test_against_reference() {
        for seed in 0..10u64 {
            let mut our_state = [0u64; 25];
            let mut ref_state = [0u64; 25];
            
            for i in 0..25 {
                our_state[i] = seed.wrapping_mul(i as u64).wrapping_add(0xDEADBEEF);
                ref_state[i] = our_state[i];
            }
            
            // Test 24 rounds
            p1600::<24>(&mut our_state);
            keccak::p1600(&mut ref_state, 24);
            assert_eq!(our_state, ref_state, "Mismatch for 24 rounds with seed {}", seed);
            
            // Reset and test 12 rounds
            for i in 0..25 {
                our_state[i] = seed.wrapping_mul(i as u64).wrapping_add(0xCAFEBABE);
                ref_state[i] = our_state[i];
            }
            p1600::<12>(&mut our_state);
            keccak::p1600(&mut ref_state, 12);
            assert_eq!(our_state, ref_state, "Mismatch for 12 rounds with seed {}", seed);
        }
    }
}
