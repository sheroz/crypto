use crate::Magma;
use crate::core::CipherBuffer;

pub struct CTR;

impl CipherBuffer for CTR {
    /// Returns encrypted result as `Vec<u8>`
    /// 
    /// Implements buffer encrypting in Counter Encryption (CTR) Mode
    /// 
    /// [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
    /// 
    /// Page 15, Section 5.2.1
    fn encrypt(core: &mut Magma, buf: &[u8]) -> Vec<u8> {
        CTR::cipher_ctr(core, buf)
    }

    /// Returns decrypted result as `Vec<u8>`
    /// 
    /// Implements buffer decrypting in Counter Encryption (CTR) Mode
    /// 
    /// [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
    /// 
    /// Page 15, Section 5.2.2
    fn decrypt(core: &mut Magma, buf: &[u8]) -> Vec<u8> {
        CTR::cipher_ctr(core, buf)
    }
}

impl CTR {
    
    /// Returns encrypted/decrypted result as `Vec<u8>`
    /// 
    /// Implements Counter Encryption (CTR) Mode
    /// 
    /// [GOST R 34.13-2015](https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf)
    /// 
    /// Page 14, Section 5.2
    fn cipher_ctr(core: &Magma, buf: &[u8]) -> Vec<u8> {

        let iv_ctr = core.prepare_vector_ctr();
        let mut result = Vec::<u8>::with_capacity(buf.len());

        for (chunk_index, chunk) in buf.chunks(8).enumerate() {
            let mut array_u8 = [0u8;8];
            chunk.iter().enumerate().for_each(|t| array_u8[t.0] = *t.1);
            let block = u64::from_be_bytes(array_u8);

            let ctr = iv_ctr.wrapping_add(chunk_index as u64);
            let gamma = core.encrypt(ctr);
            let output =  gamma ^ block;

            result.extend_from_slice(&output.to_be_bytes()[..chunk.len()]);
        }

        result
    }
}

#[cfg(test)] 
mod tests {

    use super::*;

    const CIPHER_KEY_RFC8891: [u32;8] = [
        0xffeeddcc, 0xbbaa9988, 0x77665544, 0x33221100, 0xf0f1f2f3, 0xf4f5f6f7, 0xf8f9fafb, 0xfcfdfeff
    ];

    const PLAINTEXT1_GOST_R3413_2015: u64 = 0x92def06b3c130a59_u64;
    const PLAINTEXT2_GOST_R3413_2015: u64 = 0xdb54c704f8189d20_u64;
    const PLAINTEXT3_GOST_R3413_2015: u64 = 0x4a98fb2e67a8024c_u64;
    const PLAINTEXT4_GOST_R3413_2015: u64 = 0x8912409b17b57e41_u64;

    // Test vectors GOST R 34.13-2015
    // Encrypting in CTR Mode
    // https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf
    // Page 36, Section A.2.2
    const ENCRYPTED1_CTR_GOST_R3413_2015: u64 = 0x4e98110c97b7b93c_u64;
    const ENCRYPTED2_CTR_GOST_R3413_2015: u64 = 0x3e250d93d6e85d69_u64;
    const ENCRYPTED3_CTR_GOST_R3413_2015: u64 = 0x136d868807b2dbef_u64;
    const ENCRYPTED4_CTR_GOST_R3413_2015: u64 = 0x568eb680ab52a12d_u64;      

    #[test]
    fn ctr_steps_gost_r_34_13_2015() {
        // Test vectors GOST R 34.13-2015
        // https://www.tc26.ru/standard/gost/GOST_R_3413-2015.pdf
        // Page 36, Section A.2.2

        let magma = Magma::with_key(&CIPHER_KEY_RFC8891);

        let iv = 0x12345678_u32;

        let iv_extended = (iv as u64 ) << 32;

        let mut pass_count = 0;
        let p1 = PLAINTEXT1_GOST_R3413_2015;
        let i1 = iv_extended.wrapping_add(pass_count);
        assert_eq!(i1, 0x1234567800000000_u64);
        let o1 = magma.encrypt(i1);
        assert_eq!(o1, 0xdc46e167aba4b365_u64);
        let c1 = p1 ^ o1;
        assert_eq!(c1, ENCRYPTED1_CTR_GOST_R3413_2015);

        pass_count += 1;
        let p2 = PLAINTEXT2_GOST_R3413_2015;
        let i2 = iv_extended.wrapping_add(pass_count);
        assert_eq!(i2, 0x1234567800000001_u64);
        let o2 = magma.encrypt(i2);
        assert_eq!(o2, 0xe571ca972ef0c049_u64);
        let c2 = p2 ^ o2;
        assert_eq!(c2, ENCRYPTED2_CTR_GOST_R3413_2015);

        pass_count += 1;
        let p3 = PLAINTEXT3_GOST_R3413_2015;
        let i3 = iv_extended.wrapping_add(pass_count);
        assert_eq!(i3, 0x1234567800000002_u64);
        let o3 = magma.encrypt(i3);
        assert_eq!(o3, 0x59f57da6601ad9a3_u64);
        let c3 = p3 ^ o3;
        assert_eq!(c3, ENCRYPTED3_CTR_GOST_R3413_2015);

        pass_count += 1;
        let p4 = PLAINTEXT4_GOST_R3413_2015;
        let i4 = iv_extended.wrapping_add(pass_count);
        assert_eq!(i4, 0x1234567800000003_u64);
        let o4 = magma.encrypt(i4);
        assert_eq!(o4, 0xdf9cf61bbce7df6c_u64);
        let c4 = p4 ^ o4;
        assert_eq!(c4, ENCRYPTED4_CTR_GOST_R3413_2015);
    }

    #[test]
    fn encrypt_ctr_gost_r_34_13_2015() {
        let mut source = Vec::<u8>::new();
        source.extend_from_slice(&PLAINTEXT1_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT2_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT3_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT4_GOST_R3413_2015.to_be_bytes());

        let mut magma = Magma::with_key(&CIPHER_KEY_RFC8891);
        let encrypted = CTR::encrypt(&mut magma, &source);
        assert!(!encrypted.is_empty());

        let mut expected = Vec::<u8>::new();
        expected.extend_from_slice(&ENCRYPTED1_CTR_GOST_R3413_2015.to_be_bytes());
        expected.extend_from_slice(&ENCRYPTED2_CTR_GOST_R3413_2015.to_be_bytes());
        expected.extend_from_slice(&ENCRYPTED3_CTR_GOST_R3413_2015.to_be_bytes());
        expected.extend_from_slice(&ENCRYPTED4_CTR_GOST_R3413_2015.to_be_bytes());
        assert_eq!(encrypted, expected);
    }
    
    #[test]
    fn decrypt_ctr_gost_r_34_13_2015() {
        let mut source = Vec::<u8>::new();
        source.extend_from_slice(&PLAINTEXT1_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT2_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT3_GOST_R3413_2015.to_be_bytes());
        source.extend_from_slice(&PLAINTEXT4_GOST_R3413_2015.to_be_bytes());

        let mut magma = Magma::with_key(&CIPHER_KEY_RFC8891);

        let mut encrypted = Vec::<u8>::new();
        encrypted.extend_from_slice(&ENCRYPTED1_CTR_GOST_R3413_2015.to_be_bytes());
        encrypted.extend_from_slice(&ENCRYPTED2_CTR_GOST_R3413_2015.to_be_bytes());
        encrypted.extend_from_slice(&ENCRYPTED3_CTR_GOST_R3413_2015.to_be_bytes());
        encrypted.extend_from_slice(&ENCRYPTED4_CTR_GOST_R3413_2015.to_be_bytes());

        let decrypted = CTR::decrypt(&mut magma, &encrypted);
        assert_eq!(decrypted, source);
    }    
}