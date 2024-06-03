use crate::treepp::*;
use bitcoin::{
    hashes::{ripemd160, Hash},
    key::Secp256k1,
    taproot::{TaprootBuilder, TaprootSpendInfo},
    Address, Network,
    XOnlyPublicKey,
};

use super::helper::*;

// Specialized for assert leaves currently.a
// TODO: Attach the pubkeys after constructing leaf scripts
pub type LockScript = fn(u32) -> Script;

pub type UnlockWitness = fn(u32) -> Vec<Vec<u8>>;

pub struct AssertLeaf {
    pub lock: LockScript,
    pub unlock: UnlockWitness,
}

pub fn assert_leaf() -> AssertLeaf {
  AssertLeaf {
      lock: |index| {
          script! {
              // TODO: Operator_key?
              OP_RIPEMD160
              { ripemd160::Hash::hash(format!("SECRET_{}", index).as_bytes()).as_byte_array().to_vec() }
              OP_EQUALVERIFY
              { index }
              OP_DROP
              OP_TRUE
          }
      },
      unlock: |index| vec![format!("SECRET_{}", index).as_bytes().to_vec()],
  }
}

pub fn generate_assert_leaves() -> Vec<Script> {
  // TODO: Scripts with n_of_n_pubkey and one of the commitments disprove leaves in each leaf (Winternitz signatures)
  let mut leaves = Vec::with_capacity(1000);
  let locking_template = assert_leaf().lock;
  for i in 0..1000 {
      leaves.push(locking_template(i));
  }
  leaves
}

// Returns the TaprootSpendInfo for the Commitment Taptree and the corresponding pre_sign_output
pub fn connector_c_spend_info(
  n_of_n_pubkey: XOnlyPublicKey,
) -> (TaprootSpendInfo, TaprootSpendInfo) {
  let secp = Secp256k1::new();

  let scripts = generate_assert_leaves();
  let script_weights = scripts.iter().map(|script| (1, script.clone()));
  let commitment_taptree_info = TaprootBuilder::with_huffman_tree(script_weights)
      .expect("Unable to add assert leaves")
      // Finalizing with n_of_n_pubkey allows the key-path spend with the
      // n_of_n
      .finalize(&secp, n_of_n_pubkey)
      .expect("Unable to finalize assert transaction connector c taproot");
  let pre_sign_info = TaprootBuilder::new()
      .add_leaf(0, generate_pre_sign_script(n_of_n_pubkey))
      .expect("Unable to add pre_sign script as leaf")
      .finalize(&secp, n_of_n_pubkey)
      .expect("Unable to finalize OP_CHECKSIG taproot");
  (pre_sign_info, commitment_taptree_info)
}

pub fn connector_c_address(n_of_n_pubkey: XOnlyPublicKey) -> Address {
  Address::p2tr_tweaked(
      connector_c_spend_info(n_of_n_pubkey).1.output_key(),
      Network::Testnet,
  )
}

pub fn connector_c_pre_sign_address(n_of_n_pubkey: XOnlyPublicKey) -> Address {
  Address::p2tr_tweaked(
      connector_c_spend_info(n_of_n_pubkey).0.output_key(),
      Network::Testnet,
  )
}