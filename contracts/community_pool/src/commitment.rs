use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

/// V1 preimage: domain ASCII, round u64 BE, length-prefixed ScVal address XDR,
/// bid i128 BE, and exactly 32 secret bytes. Keep this private until reveal.
const DOMAIN: &[u8; 24] = b"COMMUNITY_FINANCE_BID_V1";

pub(crate) fn hash(
    env: &Env,
    round_id: u64,
    participant: &Address,
    bid_amount: i128,
    secret: &BytesN<32>,
) -> BytesN<32> {
    let address_xdr = participant.clone().to_xdr(env);
    let mut preimage = Bytes::from_array(env, DOMAIN);
    preimage.extend_from_array(&round_id.to_be_bytes());
    preimage.extend_from_array(&address_xdr.len().to_be_bytes());
    preimage.append(&address_xdr);
    preimage.extend_from_array(&bid_amount.to_be_bytes());
    preimage.extend_from_array(&secret.to_array());
    env.crypto().sha256(&preimage).to_bytes()
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    fn from_hex<const N: usize>(hex: &str) -> [u8; N] {
        assert_eq!(hex.len(), N * 2);
        let mut bytes = [0; N];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap();
        }
        bytes
    }

    #[test]
    fn fixed_vectors_match_typescript() {
        let env = Env::default();
        let vectors = include_str!("../../../packages/shared/test-vectors/commitment-v1.txt");
        let mut hashes = std::vec::Vec::new();
        for line in vectors
            .lines()
            .filter(|line| !line.starts_with('#') && !line.is_empty())
        {
            let fields: std::vec::Vec<_> = line.split('|').collect();
            assert_eq!(fields.len(), 6);
            let round_id = fields[1].parse().unwrap();
            let participant = Address::from_str(&env, fields[2]);
            let bid_amount = fields[3].parse().unwrap();
            let secret = BytesN::from_array(&env, &from_hex::<32>(fields[4]));
            let actual = hash(&env, round_id, &participant, bid_amount, &secret).to_array();
            assert_eq!(actual, from_hex::<32>(fields[5]), "vector {}", fields[0]);
            hashes.push(actual);
        }
        assert_eq!(hashes.len(), 5);
        for changed in &hashes[1..] {
            assert_ne!(*changed, hashes[0]);
        }
    }
}
