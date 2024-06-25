use crate::cmd::*;
use helium_lib::{
    hotspot,
    keypair::{Pubkey, Signer},
};

#[derive(Clone, Debug, clap::Args)]
/// Transfer a hotspot to another owner
pub struct Cmd {
    /// Key of hotspot
    address: helium_crypto::PublicKey,
    /// Solana address of Recipient of hotspot
    recipient: Pubkey,
    /// Commit the transfer
    #[command(flatten)]
    commit: CommitOpts,
}

impl Cmd {
    pub async fn run(&self, opts: Opts) -> Result {
        let password = get_wallet_password(false)?;
        let keypair = opts.load_keypair(password.as_bytes())?;
        if keypair.pubkey() == self.recipient {
            bail!("recipient already owner of hotspot");
        }
        let client = opts.client()?;
        let tx = hotspot::transfer(&client, &self.address, &self.recipient, &keypair).await?;
        print_json(&self.commit.maybe_commit(&tx, &client).await?.to_json())
    }
}
