use crate::{
    cmd::*,
    keypair::PublicKey,
    result::{anyhow, Result},
    traits::{TxnEnvelope, TxnFee, TxnSign, TxnStakingFee, B64},
};
use serde_json::json;
use std::convert::TryInto;
use structopt::StructOpt;

/// Create or update an OUI
#[derive(Debug, StructOpt)]
pub enum Cmd {
    Create(Create),
    Submit(Submit),
}

/// Allocates an Organizational Unique Identifier (OUI) which
/// identifies endpoints for packets to sent to The transaction is not
/// submitted to the system unless the '--commit' option is given.
#[derive(Debug, StructOpt)]
pub struct Create {
    /// The address(es) of the router to send packets to
    #[structopt(long = "address", short = "a", number_of_values(1))]
    addresses: Vec<PublicKey>,

    /// Initial device membership filter in base64 encoded form
    #[structopt(long)]
    filter: String,

    /// Requested subnet size. Must be a value between 8 and 65,536
    /// and a power of two.
    #[structopt(long)]
    subnet_size: u32,

    /// Payer for the transaction (B58 address). If not specified the
    /// wallet is used.
    #[structopt(long)]
    payer: Option<PublicKey>,

    /// Commit the transaction to the API. If the staking server is
    /// used as the payer the transaction must first be submitted to
    /// the staking server for signing and the result submitted ot the
    /// API.
    #[structopt(long)]
    commit: bool,
}

/// Submits a given base64 oui transaction to the API. This command
/// can be used when this wallet is not the payer of the oui
/// transaction.
#[derive(Debug, StructOpt)]
pub struct Submit {
    /// Base64 encoded transaction to submit.
    #[structopt(name = "TRANSACTION")]
    transaction: String,

    /// Commit the payment to the API. If the staking server is used
    /// as the payer the transaction is first submitted to the staking
    /// server for signing and the result submitted ot the API.
    #[structopt(long)]
    commit: bool,
}

impl Cmd {
    pub async fn run(&self, opts: Opts) -> Result {
        match self {
            Cmd::Create(cmd) => cmd.run(opts).await,
            Cmd::Submit(cmd) => cmd.run(opts).await,
        }
    }
}

impl Create {
    pub async fn run(&self, opts: Opts) -> Result {
        let password = get_password(false)?;
        let wallet = load_wallet(opts.files)?;
        let keypair = wallet.decrypt(password.as_bytes())?;
        let wallet_key = keypair.public_key();

        let api_client = Client::new_with_base_url(api_url(wallet.public_key.network));

        let mut txn = BlockchainTxnOuiV1 {
            addresses: map_addresses(self.addresses.clone(), |v| v.to_vec())?,
            owner: keypair.public_key().into(),
            payer: self.payer.as_ref().map_or(vec![], |v| v.into()),
            oui: api_client.get_last_oui()?,
            fee: 0,
            staking_fee: 1,
            owner_signature: vec![],
            payer_signature: vec![],
            requested_subnet_size: self.subnet_size,
            filter: base64::decode(&self.filter)?,
        };
        let txn_fees = get_txn_fees(&api_client).await?;
        txn.fee = txn.txn_fee(&txn_fees)?;
        txn.staking_fee = txn.txn_staking_fee(&txn_fees)?;
        txn.owner_signature = txn.sign(&keypair)?;
        let envelope = txn.in_envelope();

        match self.payer.as_ref() {
            key if key == Some(&wallet_key) || key.is_none() => {
                // Payer is the wallet submit if ready to commit
                let status = maybe_submit_txn(self.commit, &api_client, &envelope).await?;
                print_txn(&txn, &envelope, &status, opts.format)
            }
            _ => {
                // Payer is something else.
                // can't commit this transaction but we can display it
                print_txn(&txn, &envelope, &None, opts.format)
            }
        }
    }
}

impl Submit {
    pub async fn run(&self, opts: Opts) -> Result {
        let envelope = BlockchainTxn::from_b64(&self.transaction)?;
        if let Some(Txn::Oui(t)) = envelope.txn.clone() {
            let api_url = api_url(PublicKey::from_bytes(&t.owner)?.network);
            let api_client = helium_api::Client::new_with_base_url(api_url);
            let status = maybe_submit_txn(self.commit, &api_client, &envelope).await?;
            print_txn(&t, &envelope, &status, opts.format)
        } else {
            Err(anyhow!("Invalid OUI transaction"))
        }
    }
}

fn print_txn(
    txn: &BlockchainTxnOuiV1,
    envelope: &BlockchainTxn,
    status: &Option<PendingTxnStatus>,
    format: OutputFormat,
) -> Result {
    match format {
        OutputFormat::Table => {
            ptable!(
                ["Key", "Value"],
                ["Requested OUI", txn.oui + 1],
                ["Reqeuested Subnet Size", txn.requested_subnet_size],
                [
                    "Addresses",
                    map_addresses(txn.addresses.clone(), |v| v.to_string())?.join("\n")
                ],
                ["Hash", status_str(status)]
            );

            print_footer(status)
        }
        OutputFormat::Json => {
            let table = json!({
                "requested_oui": txn.oui + 1,
                "addresses": map_addresses(txn.addresses.clone(), |v| v.to_string())?,
                "requested_subnet_size": txn.requested_subnet_size,
                "hash": status_json(status),
                "txn": envelope.to_b64()?,
            });

            print_json(&table)
        }
    }
}

fn map_addresses<F, R>(addresses: Vec<impl TryInto<PublicKey>>, f: F) -> Result<Vec<R>>
where
    F: Fn(PublicKey) -> R,
{
    let results: Result<Vec<R>> = addresses
        .into_iter()
        .map(|v| match v.try_into() {
            Ok(public_key) => Ok(f(public_key)),
            Err(_err) => Err(anyhow!("failed to convert to public key")),
        })
        .collect();
    results
}
