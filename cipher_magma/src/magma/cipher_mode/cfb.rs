use std::collections::VecDeque;

use crate::magma::Magma;

/// Returns encrypted result as `Vec<u8>`
/// 
/// Implements buffer encrypting in Cipher Feedback (CFB) Mode
/// 
/// [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
/// 
/// Page 23, Section 5.5.1
pub fn encrypt(core: &mut Magma, buf: &[u8]) -> Vec<u8> {

    core.ensure_iv_not_empty();
    let mut register_r = VecDeque::from(core.iv.clone());

    let mut result = Vec::<u8>::with_capacity(buf.len());
    for chunk in buf.chunks(8) {
        let mut array_u8 = [0u8;8];
        chunk.iter().enumerate().for_each(|t| array_u8[t.0] = *t.1);
        let block = u64::from_be_bytes(array_u8);

        let register_n= register_r.pop_front().unwrap();
        let output = core.encrypt(register_n) ^ block;

        register_r.push_back(output);

        result.extend_from_slice(&output.to_be_bytes()[..chunk.len()]);
    }

    result
}

/// Returns decrypted result as `Vec<u8>`
/// 
/// Implements buffer encrypting in Cipher Feedback (CFB) Mode
/// 
/// [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
/// 
/// Page 24, Section 5.5.2
pub fn decrypt(core: &mut Magma, buf: &[u8]) -> Vec<u8> {

    core.ensure_iv_not_empty();
    let mut register_r = VecDeque::from(core.iv.clone());

    let mut result = Vec::<u8>::with_capacity(buf.len());
    for chunk in buf.chunks(8) {
        let mut array_u8 = [0u8;8];
        chunk.iter().enumerate().for_each(|t| array_u8[t.0] = *t.1);
        let block = u64::from_be_bytes(array_u8);

        let register_n= register_r.pop_front().unwrap();
        let output = core.encrypt(register_n) ^ block;

        register_r.push_back(block);

        result.extend_from_slice(&output.to_be_bytes()[..chunk.len()]);
    }

    result
}

#[cfg(test)] 
mod tests {

    use super::*;

    #[test]
    fn cfb_steps_gost_r_34_13_2015() {
        // Test vectors GOST R 34.13-2015
        // https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf
        // Page 39, Section A.2.5

        use crypto_vectors::gost::r3413_2015;

        // s = n = 64, m = 2n = 128
        // IV = 1234567890abcdef234567890abcdef1
        let iv = 0x1234567890abcdef234567890abcdef1_u128;

        // [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
        // CFB Mode: Page 39, Section A.2.5, uses MSB(128) part of IV
        let mut r = [Magma::IV_GOST_R3413_2015[0], Magma::IV_GOST_R3413_2015[1]];
        let mut v1 = Vec::from(r[0].to_be_bytes());
        v1.extend_from_slice(&r[1].to_be_bytes());
        assert_eq!(iv.to_be_bytes(), v1.as_slice());

        let magma = Magma::with_key(&r3413_2015::CIPHER_KEY);

        let p1 = r3413_2015::PLAINTEXT1;
        let i1 = r[0];
        assert_eq!(i1, 0x1234567890abcdef_u64);
        let o1 = magma.encrypt(i1);
        assert_eq!(o1, 0x49e910895a8336da_u64);
        let c1 = o1 ^ p1;
        assert_eq!(c1, r3413_2015::CIPHERTEXT1_CFB);

        r[0] = r[1];
        r[1] = c1;

        let p2 = r3413_2015::PLAINTEXT2;
        let i2 = r[0];
        assert_eq!(i2, 0x234567890abcdef1_u64);
        let o2 = magma.encrypt(i2);
        assert_eq!(o2, 0xd612a348e78295bc_u64);
        let c2 = o2 ^ p2;
        assert_eq!(c2, r3413_2015::CIPHERTEXT2_CFB);

        r[0] = r[1];
        r[1] = c2;

        let p3 = r3413_2015::PLAINTEXT3;
        let i3 = r[0];
        assert_eq!(i3, 0xdb37e0e266903c83_u64);
        let o3 = magma.encrypt(i3);
        assert_eq!(o3, 0x6e25292d34bdd1c7_u64);
        let c3 = o3 ^ p3;
        assert_eq!(c3, r3413_2015::CIPHERTEXT3_CFB);

        r[0] = r[1];
        r[1] = c3;

        let p4 = r3413_2015::PLAINTEXT4;
        let i4 = r[0];
        assert_eq!(i4, 0x0d46644c1f9a089c_u64);
        let o4 = magma.encrypt(i4);
        assert_eq!(o4, 0x35d2728f36b22b44_u64);
        let c4 = o4 ^ p4;
        assert_eq!(c4, r3413_2015::CIPHERTEXT4_CFB);
    }

    #[test]
    fn encrypt_cfb_gost_r_34_13_2015() {
        // Test vectors GOST R 34.13-2015
        // https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf
        // Page 39, Section A.2.5

        use crypto_vectors::gost::r3413_2015;

        let mut source = Vec::<u8>::new();
        source.extend_from_slice(&r3413_2015::PLAINTEXT1.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT2.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT3.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT4.to_be_bytes());

        let mut magma = Magma::with_key(&r3413_2015::CIPHER_KEY);

        // [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
        // CFB Mode: Page 39, Section A.2.5, uses MSB(128) part of IV
        magma.set_iv(&Magma::IV_GOST_R3413_2015[..2]);

        let encrypted = encrypt(&mut magma, &source);
        assert!(!encrypted.is_empty());

        let mut expected = Vec::<u8>::new();
        expected.extend_from_slice(&r3413_2015::CIPHERTEXT1_CFB.to_be_bytes());
        expected.extend_from_slice(&r3413_2015::CIPHERTEXT2_CFB.to_be_bytes());
        expected.extend_from_slice(&r3413_2015::CIPHERTEXT3_CFB.to_be_bytes());
        expected.extend_from_slice(&r3413_2015::CIPHERTEXT4_CFB.to_be_bytes());
        assert_eq!(encrypted, expected);
    }

    #[test]
    fn decrypt_cfb_gost_r_34_13_2015() {
        // Test vectors GOST R 34.13-2015
        // https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf
        // Page 39, Section A.2.5

        use crypto_vectors::gost::r3413_2015;

        let mut source = Vec::<u8>::new();
        source.extend_from_slice(&r3413_2015::PLAINTEXT1.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT2.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT3.to_be_bytes());
        source.extend_from_slice(&r3413_2015::PLAINTEXT4.to_be_bytes());

        let mut magma = Magma::with_key(&r3413_2015::CIPHER_KEY);

        // [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
        // CFB Mode: Page 39, Section A.2.5, uses MSB(128) part of IV
        magma.set_iv(&Magma::IV_GOST_R3413_2015[..2]);

        let mut encrypted = Vec::<u8>::new();
        encrypted.extend_from_slice(&r3413_2015::CIPHERTEXT1_CFB.to_be_bytes());
        encrypted.extend_from_slice(&r3413_2015::CIPHERTEXT2_CFB.to_be_bytes());
        encrypted.extend_from_slice(&r3413_2015::CIPHERTEXT3_CFB.to_be_bytes());
        encrypted.extend_from_slice(&r3413_2015::CIPHERTEXT4_CFB.to_be_bytes());

        let decrypted = decrypt(&mut magma, &encrypted);
        assert_eq!(decrypted, source);

    }
}