# helium-wallet

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/helium/helium-wallet-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/helium/helium-wallet-rs/actions?query=workflow%3ACI+branch%3Amaster

A [Helium](https://helium.com) wallet implementation in Rust.

This is a simple wallet implementation that enables the creation and
use of an encrypted wallet.

**NOTE:** This wallet is _not_ the absolute safest way to create and
store a private key. No guarantees are implied as to its safety and
suitability for use as a wallet associated with Helium crypto-tokens.

## Installation

### From Binary

Download the latest binary for your platform here from
[Releases](https://github.com/helium/helium-wallet-rs/releases/latest). Unpack
the zip file and place the `helium-wallet` binary in your `$PATH`
somewhere.

## Usage

At any time use `-h` or `--help` to get more help for a command.

### Global options

Global options _precede_ the actual command on the command line.

The following global options are supported

* `-f` / `--file` can be used once or multiple times to specify either
  shard files for a wallet or multiple wallets if the command supports
  it. If not specified a file called `wallet.key` is assumed to be the
  wallet to use for the command.

* `--format json|table` can be used to set the output of the command
  to either a tabular format or a json output.

### Create a wallet

```
    helium-wallet create basic
```

The basic wallet will be stored in `wallet.key` after specifying an
encryption password on the command line. Options exist to specify the
wallet output file and to force overwriting an existing wallet.

Use the `--seed` option to use a previously generated
seed phrase that will be used to construct the keys for the wallet.
The app will prompt you to enter a space separated phrase. The CLI
wallet accepts 12 word or 24 word seed phrases from both Helium
mobile wallet apps as well as any valid 12 or 24 word BIP39 phrase.
Note that this does not (yet) generate an
[HD wallet](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki).

### Create a sharded wallet

Sharding wallet keys is supported via [Shamir's Secret
Sharing](https://github.com/dsprenkels/sss).  A key can be broken into
N shards such that recovering the original key needs K distinct
shards. This can be done by passing options to `create`:

```
    helium-wallet create sharded -n 5 -k 3
```

This will create wallet.key.1 through wallet.key.5 (the base name of
the wallet file can be supplied with the `-o` parameter).

When keys are sharded using `verify` will require at least K distinct
keys.

The `--seed` option described above can also be used to construct a
sharded wallet.

#### Implementation details

A ed25519 key is generated via libsodium. The provided password is run
through PBKDF2, with a configurable number of iterations and a random
salt, and the resulting value is used as an AES key. When sharding is
enabled, an additional AES key is randomly generated and the 2 keys
are combined using a sha256 HMAC into the final AES key.

The private key is then encrypted with AES256-GCM and stored in the
file along with the sharding information, the key share (if
applicable), the AES initialization vector, the PBKDF2 salt and
iteration count and the AES-GCM authentication tag.


### Public Key

```
    helium-wallet info
    helium-wallet -f my.key info
    helium-wallet -f wallet.key.1 -f wallet.key.2 -f my.key info
```

The given wallets will be read and information about the wallet,
including the public key, displayed. This command works for all wallet
types.

### Displaying

Displaying information for one or more wallets without needing its
password can be done using;


```
    helium-wallet info
```

To display a QR code for the public key of the given wallet use:

```
    helium-wallet info --qr
```

This is useful for sending tokens to the wallet from the mobile
wallet.

### Verifying

Verifying a wallet takes a password and one or more wallet files and
attempts to decrypt the wallet.

The wallet is assumed to be sharded if the first file given to the
verify command is a sharded wallet. The rest of the given files then
also have to be wallet shards. For a sharded wallet to be verified, at
least `K` wallet files must be passed in, where `K` is the value given
when creating the wallet.

```
    helium-wallet verify
    helium-wallet -f wallet.key verify
    helium-wallet -f wallet.key.1 -f wallet.key.2 -f wallet.key.5 verify
```

### Sending Tokens

#### Single Payee
To send tokens to one other account use:

```
    helium-wallet pay one <payee> <hnt>
    helium-wallet pay one <payee> <hnt> --commit
```

Where `<payee>` is the wallet address for the wallet you want to
send tokens to, `<hnt>` is the number of HNT you want to send. Since 1 HNT
is 100,000,000 bones the `hnt` value can go up to 8 decimal digits of
precision.

The default behavior of the `pay` command is to print out what the
intended payment is going to be _without_ submitting it to the
blockchain.  In the second example the `--commit` option commits the
actual payment to the API for processing by the blockchain.

#### Multiple Payees in one transaction
To send tokens to multiple other accounts use:

```
    helium-wallet pay multi <path to json file>
    helium-wallet pay multi <path to json file> --commit
```

Example json file:

```
[ { "address": "<adddress1>", "amount": <hnt1>, "memo": "<memo1>" }, { "address": "<adddress2>", "amount": <hnt2>, "memo": "<memo2>" } ]
```

Where `<address#>` is the wallet address for the wallet you want to
send tokens to, `<hnt#>` is the number of HNT you want to send. Since 1 HNT
is 100,000,000 bones the `hnt` value can go up to 8 decimal digits of
precision. `<memo#>` is an 8 byte base 64 encoded message.

The default behavior of the `pay` command is to print out what the
intended payment is going to be _without_ submitting it to the
blockchain.  In the second example the `--commit` option commits the
actual payment to the API for processing by the blockchain.


### Environment Variables

The following environment variables are supported:

* `SOLANA_MAINNET_URL` - The Solana RPC URL to use for mainnet. 
  This will get used by default or when `--url m` is passed in.
  The default mainnet URL is a rate limited API served by the Helium 
  Foundation. Use a custom provider for repeated or large requests.  

* `SOLANA_DEVNET_URL` - The Solana RPC URL to use for devnet. 
  This will get used when `--url d` is passed in.

* `HELIUM_WALLET_PASSWORD` - The password to use to decrypt the
  wallet. Useful for scripting or other non-interactive commands, but
  use with care.

* `HELIUM_WALLET_SEED_WORDS` - Space separated list of seed words to use
  when restoring a wallet from a mnemonic word list.

* `HELIUM_WALLET_SECRET` - Solana style byte array form of the keypair secret.

### Building from Source

You will need a working Rust tool-chain installed to build this CLI
from source. In addition, you will need some basic build tools.

If you wish to build from source instead of downloading
[a prebuilt release](https://github.com/helium/helium-wallet-rs/releases/latest)
you can add setup a Ubuntu 20.04 environment with the following:

```
sudo apt update
sudo apt upgrade
git clone https://github.com/helium/helium-wallet-rs
cd helium-wallet-rs
curl https://sh.rustup.rs -sSf | sh
## recommended option 1
source $HOME/.cargo/env
sudo apt install build-essential pkg-config cmake clang
```

Clone this repo:

```
git clone https://github.com/helium/helium-wallet-rs
```

and build it using cargo:

```
cd helium-wallet-rs
cargo build --release
```

The resulting `target/release/helium-wallet` is ready for use. Place
it somewhere in your `$PATH` or run it straight from the target
folder.
