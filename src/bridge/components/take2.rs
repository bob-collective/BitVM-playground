use crate::treepp::*;
use bitcoin::{
  absolute, key::Keypair, secp256k1::Message, sighash::{Prevouts, SighashCache}, taproot::LeafVersion, Amount, Sequence, TapLeafHash, TapSighashType, Transaction, TxIn, TxOut, Witness, XOnlyPublicKey
};

use super::super::context::BridgeContext;
use super::super::graph::{FEE_AMOUNT, N_OF_N_SECRET};

use super::bridge::*;
use super::connector_c::*;
use super::helper::*;
pub struct Take2Transaction {
  tx: Transaction,
  prev_outs: Vec<TxOut>,
  prev_scripts: Vec<Script>,
}

impl Take2Transaction {
  pub fn new(context: &BridgeContext, input0: Input, input1: Input, input2: Input) -> Self {
      let operator_pubkey = context
          .operator_pubkey
          .expect("operator_pubkey is required in context");
      let n_of_n_pubkey = context
          .n_of_n_pubkey
          .expect("n_of_n_pubkey is required in context");

      let _input0 = TxIn {
          previous_output: input0.0,
          script_sig: Script::new(),
          sequence: Sequence::MAX,
          witness: Witness::default(),
      };

      let _input1 = TxIn {
        previous_output: input1.0,
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::default(),
    };

    let _input2 = TxIn {
      previous_output: input2.0,
      script_sig: Script::new(),
      sequence: Sequence::MAX,
      witness: Witness::default(),
  };

  let _output0 = TxOut {
    value: input0.1 + input2.1 - Amount::from_sat(FEE_AMOUNT),
    script_pubkey: generate_pay_to_pubkey_script_address(&operator_pubkey).script_pubkey(),
};

  Take2Transaction {
          tx: Transaction {
              version: bitcoin::transaction::Version(2),
              lock_time: absolute::LockTime::ZERO,
              input: vec![_input0, _input1, _input2],
              output: vec![_output0],
          },
          prev_outs: vec![
            TxOut {
                value: input0.1,
                script_pubkey: generate_pay_to_pubkey_script_address(&n_of_n_pubkey).script_pubkey(),
            },
            TxOut {
              value: input1.1,
              script_pubkey: generate_timelock_script_address(&n_of_n_pubkey, 2).script_pubkey(),
            },
            TxOut {
                value: input2.1,
                script_pubkey: generate_pre_sign_address(&n_of_n_pubkey).script_pubkey(),
            },
        ],
        prev_scripts: vec![
          generate_pay_to_pubkey_script(&n_of_n_pubkey),
          generate_timelock_script(&n_of_n_pubkey, 2),
          generate_pre_sign_leaf0(&n_of_n_pubkey)
        ]
      }
  }

  fn pre_sign_input0(&mut self, context: &BridgeContext, operator_pubkey: &XOnlyPublicKey, n_of_n_key: &Keypair, n_of_n_pubkey: &XOnlyPublicKey) {
    let input_index = 0;

    let prevouts = Prevouts::All(&self.prev_outs);
    let prevout_leaf = (
        self.prev_scripts[input_index].clone(),
        LeafVersion::TapScript,
    );

    let sighash_type = TapSighashType::All;
    let leaf_hash =
        TapLeafHash::from_script(prevout_leaf.0.clone().as_script(), LeafVersion::TapScript);
    let mut sighash_cache = SighashCache::new(&self.tx);
    let sighash = sighash_cache
        .taproot_script_spend_signature_hash(input_index, &prevouts, leaf_hash, sighash_type)
        .expect("Failed to construct sighash");

    let signature = context.secp.sign_schnorr_no_aux_rand(&Message::from(sighash), &n_of_n_key); // This is where all n of n verifiers will sign

    let spend_info = super::connector_a::generate_spend_info(operator_pubkey, n_of_n_pubkey);
    let control_block = spend_info
        .control_block(&prevout_leaf)
        .expect("Unable to create Control block");
    self.tx.input[input_index].witness.push(bitcoin::taproot::Signature {
      signature,
      sighash_type,
    }.to_vec());
    self.tx.input[input_index].witness.push(prevout_leaf.0.to_bytes());
    self.tx.input[input_index].witness.push(control_block.serialize());
  }

  fn pre_sign_input1(&mut self, context: &BridgeContext, operator_pubkey: &XOnlyPublicKey, n_of_n_key: &Keypair, n_of_n_pubkey: &XOnlyPublicKey) {
    let input_index = 1;

    let prevouts = Prevouts::All(&self.prev_outs);
    let prevout_leaf = (
        self.prev_scripts[input_index].clone(),
        LeafVersion::TapScript,
    );

    let sighash_type = TapSighashType::All;
    let leaf_hash =
        TapLeafHash::from_script(prevout_leaf.0.clone().as_script(), LeafVersion::TapScript);
    let mut sighash_cache = SighashCache::new(&self.tx);
    let sighash = sighash_cache
        .taproot_script_spend_signature_hash(input_index, &prevouts, leaf_hash, sighash_type)
        .expect("Failed to construct sighash");

    let signature = context.secp.sign_schnorr_no_aux_rand(&Message::from(sighash), &n_of_n_key); // This is where all n of n verifiers will sign

    let spend_info = super::connector_a::generate_spend_info(operator_pubkey, n_of_n_pubkey);
    let control_block = spend_info
        .control_block(&prevout_leaf)
        .expect("Unable to create Control block");
    self.tx.input[input_index].witness.push(bitcoin::taproot::Signature {
      signature,
      sighash_type,
    }.to_vec());
    self.tx.input[input_index].witness.push(prevout_leaf.0.to_bytes());
    self.tx.input[input_index].witness.push(control_block.serialize());
  }

  fn pre_sign_input2(&mut self, context: &BridgeContext, operator_pubkey: &XOnlyPublicKey, n_of_n_key: &Keypair, n_of_n_pubkey: &XOnlyPublicKey) {
    let input_index = 2;
    let leaf_index = 0;

    let prevouts = Prevouts::All(&self.prev_outs);
    let prevout_leaf = (
        self.prev_scripts[input_index].clone(),
        LeafVersion::TapScript,
    );

    let sighash_type = TapSighashType::All;
    let leaf_hash =
        TapLeafHash::from_script(prevout_leaf.0.clone().as_script(), LeafVersion::TapScript);
    let mut sighash_cache = SighashCache::new(&self.tx);
    let sighash = sighash_cache
        .taproot_script_spend_signature_hash(leaf_index, &prevouts, leaf_hash, sighash_type)
        .expect("Failed to construct sighash");

    let signature = context.secp.sign_schnorr_no_aux_rand(&Message::from(sighash), &n_of_n_key); // This is where all n of n verifiers will sign

    let spend_info = super::connector_a::generate_spend_info(operator_pubkey, n_of_n_pubkey);
    let control_block = spend_info
        .control_block(&prevout_leaf)
        .expect("Unable to create Control block");
    self.tx.input[input_index].witness.push(bitcoin::taproot::Signature {
      signature,
      sighash_type,
    }.to_vec());
    self.tx.input[input_index].witness.push(prevout_leaf.0.to_bytes());
    self.tx.input[input_index].witness.push(control_block.serialize());
  }
}

impl BridgeTransaction for Take2Transaction {
  fn pre_sign(&mut self, context: &BridgeContext) {
    let n_of_n_key = Keypair::from_seckey_str(&context.secp, N_OF_N_SECRET).unwrap();
    let n_of_n_pubkey = context
        .n_of_n_pubkey
        .expect("n_of_n_pubkey required in context");

    let operator_pubkey = context
      .operator_pubkey
      .expect("operator_pubkey is required in context");

    self.pre_sign_input0(context, &operator_pubkey, &n_of_n_key, &n_of_n_pubkey);
    self.pre_sign_input1(context, &operator_pubkey, &n_of_n_key, &n_of_n_pubkey);
    self.pre_sign_input2(context, &operator_pubkey, &n_of_n_key, &n_of_n_pubkey);

}

fn finalize(&self, context: &BridgeContext) -> Transaction {
  self.tx.clone()
}
}