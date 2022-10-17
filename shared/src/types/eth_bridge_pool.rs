//! The necessary type definitions for the contents of the
//! Ethereum bridge pool
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use ethabi::token::Token;

use crate::ledger::eth_bridge::storage::bridge_pool::BridgePoolProof;
use crate::types::address::Address;
use crate::types::ethereum_events::{EthAddress, Uint};
use crate::types::keccak::encode::Encode;
use crate::types::keccak::KeccakHash;
use crate::types::storage::{BlockHeight, DbKeySeg, Key};
use crate::types::token::Amount;

/// A transfer message to be submitted to Ethereum
/// to move assets from Namada across the bridge.
#[derive(
    Debug,
    Clone,
    Hash,
    PartialOrd,
    PartialEq,
    Ord,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
)]
pub struct TransferToEthereum {
    /// The type of token
    pub asset: EthAddress,
    /// The recipient address
    pub recipient: EthAddress,
    /// The amount to be transferred
    pub amount: Amount,
    /// a nonce for replay protection
    pub nonce: Uint,
}

/// A transfer message to Ethereum sitting in the
/// bridge pool, waiting to be relayed
#[derive(
    Debug,
    Clone,
    Hash,
    PartialOrd,
    PartialEq,
    Ord,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
)]
pub struct PendingTransfer {
    /// The message to send to Ethereum to
    pub transfer: TransferToEthereum,
    /// The amount of gas fees (in NAM)
    /// paid by the user sending this transfer
    pub gas_fee: GasFee,
}

impl Encode<7> for PendingTransfer {
    fn tokenize(&self) -> [Token; 7] {
        let version = Token::Uint(1.into());
        let namespace = Token::String("transfer".into());
        let from = Token::String(self.gas_fee.payer.to_string());
        let fee = Token::Uint(u64::from(self.gas_fee.amount).into());
        let to = Token::Address(self.transfer.recipient.0.into());
        let amount = Token::Uint(u64::from(self.transfer.amount).into());
        let nonce = Token::Uint(self.transfer.nonce.clone().into());
        [version, namespace, from, to, amount, fee, nonce]
    }
}

impl From<&PendingTransfer> for Key {
    fn from(transfer: &PendingTransfer) -> Self {
        Key {
            segments: vec![DbKeySeg::StringSeg(
                transfer.keccak256().to_string(),
            )],
        }
    }
}

/// The amount of NAM to be payed to the relayer of
/// a transfer across the Ethereum Bridge to compensate
/// for Ethereum gas fees.
#[derive(
    Debug,
    Clone,
    Hash,
    PartialOrd,
    PartialEq,
    Ord,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
)]
pub struct GasFee {
    /// The amount of fees (in NAM)
    pub amount: Amount,
    /// The account of fee payer.
    pub payer: Address,
}

/// A Merkle root (Keccak hash) of the Ethereum
/// bridge pool that has been signed by validators'
/// Ethereum keys.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct MultiSignedMerkleRoot {
    /// The signatures from validators
    pub sigs: Vec<crate::types::key::secp256k1::Signature>,
    /// The Merkle root being signed
    pub root: KeccakHash,
    /// The block height at which this root was valid
    pub height: BlockHeight,
}

impl Encode<2> for MultiSignedMerkleRoot {
    fn tokenize(&self) -> [Token; 2] {
        let MultiSignedMerkleRoot { sigs, root, .. } = self;
        // TODO: check the tokenization of the signatures
        let sigs = Token::Array(
            sigs.iter().map(|sig| sig.tokenize()[0].clone()).collect(),
        );
        let root = Token::FixedBytes(root.0.to_vec());
        [sigs, root]
    }
}

/// All the information to relay to Ethereum
/// that a set of transfers exist in the Ethereum
/// bridge pool.
pub struct RelayProof {
    /// A merkle root signed by ta quorum of validators
    pub root: MultiSignedMerkleRoot,
    /// A membership proof
    pub proof: BridgePoolProof,
    /// A nonce for the batch for replay protection
    pub nonce: Uint,
}

impl Encode<6> for RelayProof {
    fn tokenize(&self) -> [Token; 6] {
        let [sigs, root] = self.root.tokenize();
        let [proof, transfers, flags] = self.proof.tokenize();
        [
            sigs,
            transfers,
            root,
            proof,
            flags,
            Token::Uint(self.nonce.clone().into()),
        ]
    }
}
