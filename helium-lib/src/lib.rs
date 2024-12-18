pub mod asset;
pub mod b64;
pub mod client;

pub mod boosting;
pub mod dao;
pub mod dc;
pub mod entity_key;
pub mod error;
pub mod hotspot;
pub mod keypair;
pub mod kta;
pub mod memo;
pub mod onboarding;
pub mod priority_fee;
pub mod programs;
pub mod reward;
pub mod token;

pub use anchor_client;
pub use anchor_client::solana_client;
use anchor_client::solana_client::rpc_client::SerializableTransaction;
pub use anchor_spl;
pub use helium_anchor_gen::{
    anchor_lang, circuit_breaker, data_credits, helium_entity_manager, helium_sub_daos,
    hexboosting, lazy_distributor, rewards_oracle,
};
pub use solana_sdk;
pub use solana_sdk::bs58;

pub(crate) trait Zero {
    const ZERO: Self;
}

impl Zero for u32 {
    const ZERO: Self = 0;
}

impl Zero for i32 {
    const ZERO: Self = 0;
}

impl Zero for u16 {
    const ZERO: Self = 0;
}

impl Zero for rust_decimal::Decimal {
    const ZERO: Self = rust_decimal::Decimal::ZERO;
}

pub(crate) fn is_zero<T>(value: &T) -> bool
where
    T: PartialEq + Zero,
{
    value == &T::ZERO
}

use client::SolanaRpcClient;
use error::Error;
use keypair::Pubkey;
use solana_sdk::instruction::Instruction;
use std::sync::Arc;

pub fn init(solana_client: Arc<client::SolanaRpcClient>) -> Result<(), error::Error> {
    kta::init(solana_client)
}

pub struct TransactionOpts {
    pub min_priority_fee: u64,
}

impl Default for TransactionOpts {
    fn default() -> Self {
        Self {
            min_priority_fee: priority_fee::MIN_PRIORITY_FEE,
        }
    }
}

pub struct Transaction {
    pub inner: solana_sdk::transaction::Transaction,
    pub block_height: u64,
}

impl Transaction {
    pub fn try_sign<T: solana_sdk::signers::Signers + ?Sized>(
        &mut self,
        keypairs: &T,
    ) -> Result<(), solana_sdk::signer::SignerError> {
        let recent_blockhash = self.inner.get_recent_blockhash();
        self.inner.try_sign(keypairs, *recent_blockhash)?;
        Ok(())
    }

    pub fn try_partial_sign<T: solana_sdk::signers::Signers + ?Sized>(
        &mut self,
        keypairs: &T,
    ) -> Result<(), solana_sdk::signer::SignerError> {
        let recent_blockhash = self.inner.get_recent_blockhash();
        self.inner.try_partial_sign(keypairs, *recent_blockhash)?;
        Ok(())
    }

    pub fn with_signed_transaction(self, txn: solana_sdk::transaction::Transaction) -> Self {
        Self {
            inner: txn,
            block_height: self.block_height,
        }
    }
}

pub async fn mk_transaction_with_blockhash<C: AsRef<SolanaRpcClient>>(
    client: &C,
    ixs: &[Instruction],
    payer: &Pubkey,
) -> Result<Transaction, Error> {
    let mut txn = solana_sdk::transaction::Transaction::new_with_payer(ixs, Some(payer));
    let solana_client = AsRef::<SolanaRpcClient>::as_ref(client);
    let (latest_blockhash, latest_block_height) = solana_client
        .get_latest_blockhash_with_commitment(solana_client.commitment())
        .await?;
    txn.message.recent_blockhash = latest_blockhash;
    Ok(Transaction {
        inner: txn,
        block_height: latest_block_height,
    })
}
