use crate::{cmd::*, result::Result, wallet::Wallet};
use helium_api::{accounts, models::Account};
use prettytable::Table;
use qr2term::print_qr;
use serde_json::json;

/// Get wallet information
#[derive(Debug, StructOpt)]
pub struct Cmd {
    /// Display QR code for a given single wallet.
    #[structopt(long = "qr")]
    qr_code: bool,

    /// Display basic information on a given public key
    #[structopt(long)]
    address: Option<PublicKey>,
}

impl Cmd {
    pub async fn run(&self, opts: Opts) -> Result {
        match &self.address {
            Some(public_key) => print_public_key(public_key, opts.format),
            None => {
                let wallet = load_wallet(opts.files)?;
                if self.qr_code {
                    print_qr(&wallet.public_key.to_string())?;
                    Ok(())
                } else {
                    let client = Client::new_with_base_url(api_url(wallet.public_key.network));
                    let account = accounts::get(&client, &wallet.address()?).await?;
                    print_wallet(&wallet, &account, opts.format)
                }
            }
        }
    }
}

fn print_public_key(public_key: &PublicKey, format: OutputFormat) -> Result {
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table.add_row(row!["Key", "Value"]);
            table.add_row(row!["Address", public_key.to_string()]);
            table.add_row(row!["Network", public_key.key_tag().network]);
            table.add_row(row!["Type", public_key.key_tag().key_type]);
            print_table(&table)
        }
        OutputFormat::Json => {
            let table = json!({
                "network": public_key.key_tag().network.to_string(),
                "type": public_key.key_tag().key_type.to_string(),
            });
            print_json(&table)
        }
    }
}

fn print_wallet(wallet: &Wallet, account: &Account, format: OutputFormat) -> Result {
    match format {
        OutputFormat::Table => {
            let mut table = Table::new();
            table.add_row(row!["Key", "Value"]);
            table.add_row(row!["Address", account.address]);
            table.add_row(row!["Network", wallet.public_key.key_tag().network]);
            table.add_row(row!["Type", wallet.public_key.key_tag().key_type]);
            table.add_row(row!["Sharded", wallet.is_sharded()]);
            table.add_row(row!["PwHash", wallet.pwhash()]);
            table.add_row(row!["Balance", account.balance]);
            table.add_row(row!["DC Balance", account.dc_balance]);
            table.add_row(row!["Securities Balance", account.sec_balance]);
            print_table(&table)
        }
        OutputFormat::Json => {
            let table = json!({
                "sharded": wallet.is_sharded(),
                "network": wallet.public_key.key_tag().network.to_string(),
                "type": wallet.public_key.key_tag().key_type.to_string(),
                "pwhash": wallet.pwhash().to_string(),
                "account": account,
            });
            print_json(&table)
        }
    }
}
