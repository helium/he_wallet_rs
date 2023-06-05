use crate::{
    cmd::{load_wallet, print_json, Opts},
    result::{Error, Result},
    wallet::Wallet,
};
use qr2term::print_qr;
use serde_json::json;

/// Get wallet information
#[derive(Debug, clap::Args)]
pub struct Cmd {
    /// Display QR code for a given single wallet.
    #[arg(long)]
    qr: bool,
}

impl Cmd {
    pub fn run(&self, opts: Opts) -> Result {
        let wallet = load_wallet(&opts.files)?;
        if self.qr {
            print_qr(wallet.public_key.to_string()).map_err(Error::from)
        } else {
            print_wallet(&wallet)
        }
    }
}

pub(crate) fn print_wallet(wallet: &Wallet) -> Result {
    let json = json!({
        "sharded": wallet.is_sharded(),
        "pwhash": wallet.pwhash().to_string(),
        "address": wallet.public_key.to_string(),
    });
    print_json(&json)
}
