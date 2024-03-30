use crate::bigint::add::u30_add_carry;
use crate::bigint::sub::u30_sub_borrow;
use crate::bigint::{MAX_U30, U254};
use crate::treepp::*;

pub struct Fq;

// "inherit methods from BigInt"
impl Fq {
    #[inline]
    pub fn copy(a: u32) -> Script { U254::copy(a) }

    #[inline]
    pub fn roll(a: u32) -> Script { U254::roll(a) }

    #[inline]
    pub fn drop() -> Script { U254::drop() }

    #[inline]
    pub fn zip(a: u32, b: u32) -> Script { U254::zip(a, b) }

    #[inline]
    pub fn push_u32_le(v: &[u32]) -> Script { U254::push_u32_le(v) }

    #[inline]
    pub fn equalverify(a: u32, b: u32) -> Script { U254::equalverify(a, b) }

    #[inline]
    pub fn push_hex(hex_string: &str) -> Script { U254::push_hex(hex_string) }
}

impl Fq {
    const MODULUS: &'static str =
        "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47";

    const N_LIMBS: u32 = U254::N_LIMBS;
    const N_BITS: u32 = U254::N_BITS;

    pub fn push_modulus() -> Script { Fq::push_hex(Fq::MODULUS) }

    pub fn push_zero() -> Script {
        script! {
            for _ in 0..Fq::N_LIMBS { 0 }
        }
    }

    pub fn push_one() -> Script {
        script! {
            for _ in 1..Fq::N_LIMBS { 0 }
            1
        }
    }

    // A + B mod M
    // Ci⁺ overflow carry bit (A+B)
    // Ci⁻ overflow carry bit (A-B)
    pub fn add(a: u32, b: u32) -> Script {
        // Modulus as 30-bit limbs
        let modulus = [
            0x187CFD47, 0x3082305B, 0x71CA8D3, 0x205AA45A, 0x1585D97, 0x116DA06, 0x1A029B85,
            0x139CB84C, 0x3064,
        ];
        script! {
            { Fq::zip(a, b) }

            { MAX_U30 }

            // A₀ + B₀
            u30_add_carry
            // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ B₁ 2³⁰ C₀⁺ A₀+B₀
            OP_DUP
            OP_TOALTSTACK
            // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ B₁ 2³⁰ C₀⁺ A₀+B₀ | A₀+B₀
            OP_ROT
            // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ B₁ C₀⁺ A₀+B₀ 2³⁰
            { modulus[0] }
            OP_SWAP
            // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ B₁ C₀⁺ A₀+B₀ M₀ 2³⁰
            u30_sub_borrow
            OP_TOALTSTACK
            // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ B₁ C₀⁺ 2³⁰ C₀⁻ | (A₀+B₀)-M₀

            // from     A₁      + B₁        + carry_0
            //   to     A{N-2}  + B{N-2}    + carry_{N-3}
            for i in 1..Fq::N_LIMBS-1 {
                OP_2SWAP
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ 2³⁰ C₀⁻ B₁ C₀⁺
                OP_ADD
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ A₁ 2³⁰ C₀⁻ B₁+C₀⁺
                OP_2SWAP
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₀⁻ B₁+C₀⁺ A₁ 2³⁰
                u30_add_carry
                OP_DUP
                OP_TOALTSTACK
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₀⁻ 2³⁰ C₁⁺ (B₁+C₀)+A₁ | (B₁+C₀)+A₁
                OP_2SWAP
                OP_SWAP
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₁⁺ (B₁+C₀)+A₁ 2³⁰ C₀⁻
                { modulus[i as usize] }
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₁⁺ (B₁+C₀)+A₁ 2³⁰ C₀⁻ M₁
                OP_ADD
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₁⁺ (B₁+C₀)+A₁ 2³⁰ C₀⁻+M₁
                OP_ROT
                OP_SWAP
                OP_ROT
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₁⁺ (B₁+C₀)+A₁ C₀⁻+M₁ 2³⁰
                u30_sub_borrow
                OP_TOALTSTACK
                // A₈ B₈ A₇ B₇ A₆ B₆ A₅ B₅ A₄ B₄ A₃ B₃ A₂ B₂ C₁⁺ 2³⁰ C₁⁻ | ((B₁+C₀)+A₁)-(C₀⁻+M₁)
            }
            // A₈ B₈ C₇⁺ 2³⁰ C₇⁻
            OP_2SWAP
            OP_ADD
            // A₈ 2³⁰ C₇⁻ B₈+C₇⁺
            OP_2SWAP
            OP_ROT
            OP_ROT
            // C₇⁻ 2³⁰ B₈+C₇⁺ A₈
            OP_ADD
            // C₇⁻ 2³⁰ (B₈+C₇⁺)+A₈
            OP_DUP
            OP_TOALTSTACK
            OP_ROT
            // 2³⁰ (B₈+C₇⁺)+A₈ C₇⁻
            { *modulus.last().unwrap() }
            // 2³⁰ (B₈+C₇⁺)+A₈ C₇⁻ M₈
            OP_ADD
            OP_ROT
            // (B₈+C₇⁺)+A₈ C₇⁻+M₈ 2³⁰
            u30_sub_borrow
            OP_TOALTSTACK
            // 2³⁰ C₈⁻ | ((B₈+C₇⁺)+A₈)-(C₇⁻+M₈)
            OP_NIP
            OP_DUP
            // C₈⁻ C₈⁻
            OP_IF
                OP_FROMALTSTACK
                OP_DROP
            OP_ENDIF

            OP_FROMALTSTACK
            OP_SWAP
            // (B₈+C₇⁺)+A₈ C₈⁻ | ((B₇+C₆⁺)+A₇)-(C₆⁻+M₇)
            // ((B₈+C₇⁺)+A₈)-(C₇⁻+M₈) C₈⁻ | (B₈+C₇⁺)+A₈
            for _ in 0..Fq::N_LIMBS-1 {
                OP_FROMALTSTACK  OP_DROP
                OP_FROMALTSTACK  OP_SWAP
            }
            // (B₈+C₇⁺)+A₈ (B₇+C₆⁺)+A₇ ... (B₂+C₁⁺)+A₂ (B₁+C₀⁺)+A₁ A₀+B₀ C₈⁻
            // ((B₈+C₇⁺)+A₈)-(C₇⁻+M₈) ... (A₀+B₀)-M₀ C₈⁻ | A₀+B₀
            OP_NOT
            OP_IF
                OP_FROMALTSTACK
                OP_DROP
            OP_ENDIF
            // (B₈+C₇⁺)+A₈ (B₇+C₆⁺)+A₇ ... (B₁+C₀⁺)+A₁ A₀+B₀
            // ((B₈+C₇⁺)+A₈)-(C₇⁻+M₈) ... (A₀+B₀)-M₀
        }
    }

    pub fn neg(a: u32) -> Script {
        script! {
            { Fq::push_modulus() }
            { U254::sub(0, a + 1) }
        }
    }

    pub fn sub(a: u32, b: u32) -> Script {
        script! {
            { Fq::neg(b) }
            if a > b {
                { Fq::add(0, a) }
            } else {
                { Fq::add(0, a + 1) }
            }
        }
    }

    pub fn double(a: u32) -> Script {
        script! {
            { Fq::copy(a) }
            { Fq::add(a + 1, 0) }
        }
    }

    pub fn mul() -> Script {
        script! {
            { U254::convert_to_bits_toaltstack() }

            { Fq::push_zero() }

            OP_FROMALTSTACK
            OP_IF
                { Fq::copy(1) }
                { Fq::add(1, 0) }
            OP_ENDIF

            for _ in 1..Fq::N_BITS - 1 {
                { Fq::roll(1) }
                { Fq::double(0) }
                { Fq::roll(1) }
                OP_FROMALTSTACK
                OP_IF
                    { Fq::copy(1) }
                    { Fq::add(1, 0) }
                OP_ENDIF
            }

            { Fq::roll(1) }
            { Fq::double(0) }
            OP_FROMALTSTACK
            OP_IF
                { Fq::add(1, 0) }
            OP_ELSE
                { Fq::drop() }
            OP_ENDIF
        }
    }

    pub fn square() -> Script {
        script! {
            { Fq::copy(0) }
            { Fq::mul() }
        }
    }

    pub fn inv() -> Script {
        script! {
            { Fq::push_modulus() }
            { Fq::roll(1) }
            { U254::inv_stage1() }
            { U254::inv_stage2(Self::MODULUS) }
            { Fq::mul() }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::bn254::fq::Fq;
    use crate::treepp::*;
    use ark_ff::Field;
    use ark_std::UniformRand;
    use core::ops::{Add, Mul, Rem, Sub};
    use num_bigint::{BigUint, RandomBits};
    use num_traits::Num;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_add() {
        println!("Fq.add: {} bytes", Fq::add(0, 1).len());

        let m = BigUint::from_str_radix(Fq::MODULUS, 16).unwrap();

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a: BigUint = prng.sample(RandomBits::new(254));
            let b: BigUint = prng.sample(RandomBits::new(254));

            let a = a.rem(&m);
            let b = b.rem(&m);
            let c: BigUint = a.clone().add(b.clone()).rem(&m);

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::push_u32_le(&b.to_u32_digits()) }
                { Fq::add(1, 0) }
                { Fq::push_u32_le(&c.to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_sub() {
        println!("Fq.sub: {} bytes", Fq::sub(0, 1).len());

        let m = BigUint::from_str_radix(Fq::MODULUS, 16).unwrap();

        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..100 {
            let a: BigUint = prng.sample(RandomBits::new(254));
            let b: BigUint = prng.sample(RandomBits::new(254));

            let a = a.rem(&m);
            let b = b.rem(&m);
            let c: BigUint = a.clone().add(&m).sub(b.clone()).rem(&m);

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::push_u32_le(&b.to_u32_digits()) }
                { Fq::sub(1, 0) }
                { Fq::push_u32_le(&c.to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_double() {
        println!("Fq.double: {} bytes", Fq::double(0).len());
        let m = BigUint::from_str_radix(Fq::MODULUS, 16).unwrap();

        for _ in 0..100 {
            let a: BigUint = m.clone().sub(BigUint::new(vec![1]));

            let a = a.rem(&m);
            let c: BigUint = a.clone().add(a.clone()).rem(&m);

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::double(0) }
                { Fq::push_u32_le(&c.to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_mul() {
        println!("Fq.mul: {} bytes", Fq::mul().len());
        let m = BigUint::from_str_radix(Fq::MODULUS, 16).unwrap();
        let mut prng = ChaCha20Rng::seed_from_u64(0);
        for _ in 0..3 {
            let a: BigUint = prng.sample(RandomBits::new(254));
            let b: BigUint = prng.sample(RandomBits::new(254));

            let a = a.rem(&m);
            let b = b.rem(&m);
            let c: BigUint = a.clone().mul(b.clone()).rem(&m);

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::push_u32_le(&b.to_u32_digits()) }
                { Fq::mul() }
                { Fq::push_u32_le(&c.to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_square() {
        println!("Fq.square: {} bytes", Fq::square().len());
        let m = BigUint::from_str_radix(Fq::MODULUS, 16).unwrap();

        let mut prng = ChaCha20Rng::seed_from_u64(0);
        for _ in 0..3 {
            let a: BigUint = prng.sample(RandomBits::new(254));

            let a = a.rem(&m);
            let c: BigUint = a.clone().mul(a.clone()).rem(&m);

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::square() }
                { Fq::push_u32_le(&c.to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_neg() {
        println!("Fq.neg: {} bytes", Fq::neg(0).len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..3 {
            let a: BigUint = prng.sample(RandomBits::new(254));

            let script = script! {
                { Fq::push_u32_le(&a.to_u32_digits()) }
                { Fq::copy(0) }
                { Fq::neg(0) }
                { Fq::add(0, 1) }
                { Fq::push_zero() }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }

    #[test]
    fn test_inv() {
        println!("Fq.inv: {} bytes", Fq::inv().len());
        let mut prng = ChaCha20Rng::seed_from_u64(0);

        for _ in 0..1 {
            let a = ark_bn254::Fq::rand(&mut prng);
            let c = a.inverse().unwrap();

            let script = script! {
                { Fq::push_u32_le(&BigUint::from(a).to_u32_digits()) }
                { Fq::inv() }
                { Fq::push_u32_le(&BigUint::from(c).to_u32_digits()) }
                { Fq::equalverify(1, 0) }
                OP_TRUE
            };
            let exec_result = execute_script(script);
            assert!(exec_result.success);
        }
    }
}