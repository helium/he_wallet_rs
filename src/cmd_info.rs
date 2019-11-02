use crate::{
    traits::{ReadWrite, B58},
    wallet::Wallet,
};
use prettytable::Table;
use std::{error::Error, fs, path::PathBuf, result::Result};

pub fn cmd_info(files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let mut wallets = Vec::with_capacity(files.len());
    for file in files {
        let mut reader = fs::File::open(&file)?;
        let enc_wallet = Wallet::read(&mut reader)?;
        wallets.push(enc_wallet);
    }
    print_wallets(wallets);
    Ok(())
}

fn print_wallets(wallets: Vec<Wallet>) {
    let mut table = Table::new();

    table.add_row(row!["Address", "Sharded"]);
    for wallet in wallets {
        let address = wallet.public_key().to_b58().unwrap_or("unknown".to_string());
        table.add_row(row![address, wallet.is_sharded()]);
    }
    table.printstd();
}
