use crate::{cmd::*, result::Result};

mod stake;
mod transfer;
mod unstake;

#[derive(Debug, StructOpt)]
/// Commands for validators
pub enum Cmd {
    // List validators for the given wallet.
    // List(List),
    /// Stake a validator with the given wallet as the owner.
    Stake(stake::Cmd),
    /// Unstake a validator
    Unstake(unstake::Cmd),
    /// Transfer a validator stake to a new validator and owner
    Transfer(Box<transfer::Cmd>),
}

impl Cmd {
    pub async fn run(self, opts: Opts) -> Result {
        match self {
            // Self::List(cmd) => cmd.run(opts),
            Self::Stake(cmd) => cmd.run(opts).await,
            Self::Unstake(cmd) => cmd.run(opts).await,
            Self::Transfer(cmd) => cmd.run(opts).await,
        }
    }
}
