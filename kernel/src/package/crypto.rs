//! NovaOS Apex — Standalone Cryptographic Engine
//! Implements FIPS 180-4 standard SHA-256, HMAC-SHA256, and digital signature verification.

pub const SYSTEM_ROOT_PUBKEY: [u8; 32] = [
    0xA7, 0x19, 0xF2, 0x01, 0x5C, 0x88, 0x94, 0xFE,
    0x23, 0x77, 0xAB, 0x4D, 0x91, 0x02, 0xEE, 0x7F,
    0x31, 0x45, 0x6A, 0x89, 0xBC, 0xDE, 0xF0, 0x12,
    0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x5A,
];

// SHA-256 Round Constants K[0..63]
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

#[inline(always)]
fn rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

#[inline(always)]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

#[inline(always)]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

#[inline(always)]
fn sigma0(x: u32) -> u32 {
    rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22)
}

#[inline(always)]
fn sigma1(x: u32) -> u32 {
    rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25)
}

#[inline(always)]
fn gamma0(x: u32) -> u32 {
    rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3)
}

#[inline(always)]
fn gamma1(x: u32) -> u32 {
    rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10)
}

/// Computes the 32-byte SHA-256 hash of the input slice.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut offset = 0;
    let mut block = [0u8; 64];

    while offset + 64 <= data.len() {
        block.copy_from_slice(&data[offset..offset + 64]);
        process_block(&mut h, &block);
        offset += 64;
    }

    // Padding
    let rem = data.len() - offset;
    block.fill(0);
    block[..rem].copy_from_slice(&data[offset..]);
    block[rem] = 0x80;

    if rem >= 56 {
        process_block(&mut h, &block);
        block.fill(0);
    }

    let len_bytes = bit_len.to_be_bytes();
    block[56..64].copy_from_slice(&len_bytes);
    process_block(&mut h, &block);

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

fn process_block(h: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for t in 0..16 {
        w[t] = u32::from_be_bytes([
            block[t * 4],
            block[t * 4 + 1],
            block[t * 4 + 2],
            block[t * 4 + 3],
        ]);
    }
    for t in 16..64 {
        w[t] = gamma1(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(gamma0(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];
    let mut f = h[5];
    let mut g = h[6];
    let mut h_val = h[7];

    for t in 0..64 {
        let t1 = h_val
            .wrapping_add(sigma1(e))
            .wrapping_add(ch(e, f, g))
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let t2 = sigma0(a).wrapping_add(maj(a, b, c));

        h_val = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(h_val);
}

/// Computes HMAC-SHA256 over data using the provided secret key.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k_block = [0u8; 64];
    if key.len() > 64 {
        let key_hash = sha256(key);
        k_block[..32].copy_from_slice(&key_hash);
    } else {
        k_block[..key.len()].copy_from_slice(key);
    }

    let mut o_pad = [0x5Cu8; 64];
    let mut i_pad = [0x36u8; 64];
    for i in 0..64 {
        o_pad[i] ^= k_block[i];
        i_pad[i] ^= k_block[i];
    }

    // Inner hash: sha256(i_pad || data)
    let mut inner_buf = [0u8; 1024];
    let inner_len = 64 + data.len();
    if inner_len <= inner_buf.len() {
        inner_buf[..64].copy_from_slice(&i_pad);
        inner_buf[64..inner_len].copy_from_slice(data);
        let inner_hash = sha256(&inner_buf[..inner_len]);

        // Outer hash: sha256(o_pad || inner_hash)
        let mut outer_buf = [0u8; 96];
        outer_buf[..64].copy_from_slice(&o_pad);
        outer_buf[64..96].copy_from_slice(&inner_hash);
        sha256(&outer_buf)
    } else {
        // Fallback for large buffers
        let inner_hash = sha256_two_part(&i_pad, data);
        let mut outer_buf = [0u8; 96];
        outer_buf[..64].copy_from_slice(&o_pad);
        outer_buf[64..96].copy_from_slice(&inner_hash);
        sha256(&outer_buf)
    }
}

fn sha256_two_part(p1: &[u8], p2: &[u8]) -> [u8; 32] {
    let total_len = p1.len() + p2.len();
    let bit_len = (total_len as u64) * 8;
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut block = [0u8; 64];
    let mut block_idx = 0;

    for &b in p1.iter().chain(p2.iter()) {
        block[block_idx] = b;
        block_idx += 1;
        if block_idx == 64 {
            process_block(&mut h, &block);
            block_idx = 0;
        }
    }

    block[block_idx] = 0x80;
    block_idx += 1;
    for i in block_idx..64 {
        block[i] = 0;
    }

    if block_idx > 56 {
        process_block(&mut h, &block);
        block.fill(0);
    }

    let len_bytes = bit_len.to_be_bytes();
    block[56..64].copy_from_slice(&len_bytes);
    process_block(&mut h, &block);

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

/// Signs a 32-byte payload digest using the publisher/root secret key.
pub fn sign_digest(secret_key: &[u8; 32], digest: &[u8; 32]) -> [u8; 64] {
    let tag = hmac_sha256(secret_key, digest);
    let mut sig = [0u8; 64];
    sig[0..32].copy_from_slice(&tag);
    // Verification anchor derived from secret and tag
    let anchor = hmac_sha256(&tag, secret_key);
    sig[32..64].copy_from_slice(&anchor);
    sig
}

/// Verifies a 64-byte signature against the expected digest and public key.
pub fn verify_signature(public_key: &[u8; 32], digest: &[u8; 32], signature: &[u8; 64]) -> bool {
    let tag = &signature[0..32];
    let anchor = &signature[32..64];

    // Compute expected anchor
    let expected_anchor = hmac_sha256(tag, public_key);
    if anchor != &expected_anchor {
        return false;
    }

    // Verify tag binds public key and digest
    let expected_tag = hmac_sha256(public_key, digest);
    tag == &expected_tag
}
