// Note: The substrate build requires these be imported
pub use our_std::vec::Vec;

use crate::rates::APR;
use crate::reason::Reason;
use crate::types::{AssetAmount, CashIndex, Timestamp};

use codec::{Decode, Encode};
use compound_crypto::public_key_bytes_to_eth_address;
use our_std::{convert::TryInto, Debuggable, Deserialize, RuntimeDebug, Serialize};

/// Type for representing the selection of a supported chain.
#[derive(Serialize, Deserialize)] // used in config
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainId {
    Comp,
    Eth,
    Dot,
    Sol,
    Tez,
}

impl ChainId {
    pub fn to_account(self, addr: &str) -> Result<ChainAccount, Reason> {
        match self {
            ChainId::Comp => Ok(ChainAccount::Comp(Compound::to_address(addr)?)),
            ChainId::Eth => Ok(ChainAccount::Eth(Ethereum::to_address(addr)?)),
            ChainId::Dot => Ok(ChainAccount::Dot(Polkadot::to_address(addr)?)),
            ChainId::Sol => Ok(ChainAccount::Sol(Solana::to_address(addr)?)),
            ChainId::Tez => Ok(ChainAccount::Tez(Tezos::to_address(addr)?)),
        }
    }

    pub fn to_asset(self, addr: &str) -> Result<ChainAsset, Reason> {
        match self {
            ChainId::Comp => Ok(ChainAsset::Comp(Compound::to_address(addr)?)),
            ChainId::Eth => Ok(ChainAsset::Eth(Ethereum::to_address(addr)?)),
            ChainId::Dot => Ok(ChainAsset::Dot(Polkadot::to_address(addr)?)),
            ChainId::Sol => Ok(ChainAsset::Sol(Solana::to_address(addr)?)),
            ChainId::Tez => Ok(ChainAsset::Tez(Tezos::to_address(addr)?)),
        }
    }

    pub fn signer_address(self) -> Result<ChainAccount, Reason> {
        match self {
            ChainId::Comp => Ok(ChainAccount::Comp(<Compound as Chain>::signer_address()?)),
            ChainId::Eth => Ok(ChainAccount::Eth(<Ethereum as Chain>::signer_address()?)),
            ChainId::Dot => Ok(ChainAccount::Dot(<Polkadot as Chain>::signer_address()?)),
            ChainId::Sol => Ok(ChainAccount::Sol(<Solana as Chain>::signer_address()?)),
            ChainId::Tez => Ok(ChainAccount::Tez(<Tezos as Chain>::signer_address()?)),
        }
    }

    pub fn hash_bytes(self, data: &[u8]) -> ChainHash {
        match self {
            ChainId::Comp => ChainHash::Comp(<Compound as Chain>::hash_bytes(data)),
            ChainId::Eth => ChainHash::Eth(<Ethereum as Chain>::hash_bytes(data)),
            ChainId::Dot => ChainHash::Dot(<Polkadot as Chain>::hash_bytes(data)),
            ChainId::Sol => ChainHash::Sol(<Solana as Chain>::hash_bytes(data)),
            ChainId::Tez => ChainHash::Tez(<Tezos as Chain>::hash_bytes(data)),
        }
    }

    pub fn sign(self, message: &[u8]) -> Result<ChainSignature, Reason> {
        match self {
            ChainId::Comp => Ok(ChainSignature::Comp(<Compound as Chain>::sign_message(
                message,
            )?)),
            ChainId::Eth => Ok(ChainSignature::Eth(<Ethereum as Chain>::sign_message(
                message,
            )?)),
            ChainId::Dot => Ok(ChainSignature::Dot(<Polkadot as Chain>::sign_message(
                message,
            )?)),
            ChainId::Sol => Ok(ChainSignature::Sol(<Solana as Chain>::sign_message(
                message,
            )?)),
            ChainId::Tez => Ok(ChainSignature::Tez(<Tezos as Chain>::sign_message(
                message,
            )?)),
        }
    }

    pub fn zero_hash(self) -> ChainHash {
        match self {
            ChainId::Comp => ChainHash::Comp(<Compound as Chain>::zero_hash()),
            ChainId::Eth => ChainHash::Eth(<Ethereum as Chain>::zero_hash()),
            ChainId::Dot => ChainHash::Dot(<Polkadot as Chain>::zero_hash()),
            ChainId::Sol => ChainHash::Sol(<Solana as Chain>::zero_hash()),
            ChainId::Tez => ChainHash::Tez(<Tezos as Chain>::zero_hash()),
        }
    }
}

impl Default for ChainId {
    fn default() -> Self {
        ChainId::Eth
    }
}

/// Type for an account tied to a chain.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainAccount {
    Comp(<Compound as Chain>::Address),
    Eth(<Ethereum as Chain>::Address),
    Dot(<Polkadot as Chain>::Address),
    Sol(<Solana as Chain>::Address),
    Tez(<Tezos as Chain>::Address),
}

impl ChainAccount {
    pub fn chain_id(&self) -> ChainId {
        match *self {
            ChainAccount::Eth(_) => ChainId::Eth,
            _ => panic!("XXX not implemented"),
        }
    }
}

/// Type for an hash tied to a chain.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainHash {
    Comp(<Compound as Chain>::Hash),
    Eth(<Ethereum as Chain>::Hash),
    Dot(<Polkadot as Chain>::Hash),
    Sol(<Solana as Chain>::Hash),
    Tez(<Tezos as Chain>::Hash),
}

// Implement deserialization for ChainAccounts so we can use them in GenesisConfig / ChainSpec JSON.
//  i.e. "eth:0x..." <> Eth(0x...)
impl our_std::str::FromStr for ChainAccount {
    type Err = Reason;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some((chain_id_str, address_str)) = String::from(string).split_once(":") {
            let chain_id = ChainId::from_str(chain_id_str)?;
            Ok(chain_id.to_account(address_str)?)
        } else {
            Err(Reason::BadAsset)
        }
    }
}

// For serialize (which we don't really use, but are required to implement)
impl From<ChainAccount> for String {
    fn from(asset: ChainAccount) -> String {
        match asset {
            ChainAccount::Eth(address) => format!("ETH:0x{}", hex::encode(address)),
            _ => panic!("XXX not implemented"),
        }
    }
}

/// Type for an asset or CASH
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum CashAsset {
    Cash,
    Asset(ChainAsset),
}

/// Type for an asset tied to a chain.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainAsset {
    Comp(<Compound as Chain>::Address),
    Eth(<Ethereum as Chain>::Address),
    Dot(<Polkadot as Chain>::Address),
    Sol(<Solana as Chain>::Address),
    Tez(<Tezos as Chain>::Address),
}

// For serialize (which we don't really use, but are required to implement)
impl ChainAsset {
    pub fn chain_id(&self) -> ChainId {
        match *self {
            ChainAsset::Eth(_) => ChainId::Eth,
            _ => panic!("XXX not implemented"),
        }
    }
}

// Implement deserialization for ChainAssets so we can use them in GenesisConfig / ChainSpec JSON.
//  i.e. "eth:0x..." <> Eth(0x...)
impl our_std::str::FromStr for ChainAsset {
    type Err = Reason;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some((chain_id_str, address_str)) = String::from(string).split_once(":") {
            let chain_id = ChainId::from_str(chain_id_str)?;
            Ok(chain_id.to_asset(address_str)?)
        } else {
            Err(Reason::BadAsset)
        }
    }
}

impl From<ChainAsset> for String {
    fn from(asset: ChainAsset) -> String {
        match asset {
            ChainAsset::Eth(address) => format!("ETH:0x{}", hex::encode(address)),
            _ => panic!("XXX not implemented"),
        }
    }
}

/// Type for chain assets paired with an account
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainAssetAccount {
    Comp(<Compound as Chain>::Address, <Compound as Chain>::Address),
    Eth(<Ethereum as Chain>::Address, <Ethereum as Chain>::Address),
    Dot(<Polkadot as Chain>::Address, <Polkadot as Chain>::Address),
    Sol(<Solana as Chain>::Address, <Solana as Chain>::Address),
    Tez(<Tezos as Chain>::Address, <Tezos as Chain>::Address),
}

/// Type for a signature and account tied to a chain.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainAccountSignature {
    Comp(<Compound as Chain>::Address, <Compound as Chain>::Signature),
    Eth(<Ethereum as Chain>::Address, <Ethereum as Chain>::Signature),
    Dot(<Polkadot as Chain>::Address, <Polkadot as Chain>::Signature),
    Sol(<Solana as Chain>::Address, <Solana as Chain>::Signature),
    Tez(<Tezos as Chain>::Address, <Tezos as Chain>::Signature),
}

impl ChainAccountSignature {
    pub fn to_chain_signature(self) -> ChainSignature {
        match self {
            ChainAccountSignature::Comp(_, sig) => ChainSignature::Comp(sig),
            ChainAccountSignature::Eth(_, sig) => ChainSignature::Eth(sig),
            ChainAccountSignature::Dot(_, sig) => ChainSignature::Dot(sig),
            ChainAccountSignature::Sol(_, sig) => ChainSignature::Sol(sig),
            ChainAccountSignature::Tez(_, sig) => ChainSignature::Tez(sig),
        }
    }

    pub fn recover_account(self, message: &[u8]) -> Result<ChainAccount, Reason> {
        match self {
            ChainAccountSignature::Eth(eth_account, eth_sig) => {
                let recovered = <Ethereum as Chain>::recover_address(message, eth_sig)?;
                if eth_account == recovered {
                    Ok(ChainAccount::Eth(recovered))
                } else {
                    Err(Reason::SignatureAccountMismatch)
                }
            }
            _ => panic!("XXX not implemented"),
        }
    }
}

/// Type for a signature tied to a chain.
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainSignature {
    Comp(<Compound as Chain>::Signature),
    Eth(<Ethereum as Chain>::Signature),
    Dot(<Polkadot as Chain>::Signature),
    Sol(<Solana as Chain>::Signature),
    Tez(<Tezos as Chain>::Signature),
}

impl ChainSignature {
    pub fn chain_id(&self) -> ChainId {
        match *self {
            ChainSignature::Eth(_) => ChainId::Eth,
            _ => panic!("XXX not implemented"),
        }
    }

    pub fn recover(&self, message: &[u8]) -> Result<ChainAccount, Reason> {
        match self {
            ChainSignature::Eth(eth_sig) => Ok(ChainAccount::Eth(
                <Ethereum as Chain>::recover_address(message, *eth_sig)?,
            )),

            _ => panic!("XXX not implemented"),
        }
    }
}

/// Type for a list of chain signatures.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum ChainSignatureList {
    Comp(Vec<(<Compound as Chain>::Address, <Compound as Chain>::Signature)>),
    Eth(Vec<(<Ethereum as Chain>::Address, <Ethereum as Chain>::Signature)>),
    Dot(Vec<(<Polkadot as Chain>::Address, <Polkadot as Chain>::Signature)>),
    Sol(Vec<(<Solana as Chain>::Address, <Solana as Chain>::Signature)>),
    Tez(Vec<(<Tezos as Chain>::Address, <Tezos as Chain>::Signature)>),
}

// Implement deserialization for ChainIds so we can use them in GenesisConfig / ChainSpec JSON.
impl our_std::str::FromStr for ChainId {
    type Err = Reason;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "ETH" => Ok(ChainId::Eth),
            "SOL" => Ok(ChainId::Sol),
            _ => Err(Reason::BadChainId),
        }
    }
}

pub trait Chain {
    const ID: ChainId;

    type Address: Debuggable + Clone + Eq + Into<Vec<u8>> = [u8; 20];
    type Amount: Debuggable + Clone + Eq + Into<AssetAmount> = u128;
    type CashIndex: Debuggable + Clone + Eq + Into<CashIndex> = u128;
    type Rate: Debuggable + Clone + Eq + Into<APR> = u128;
    type Timestamp: Debuggable + Clone + Eq + Into<Timestamp> = u128; // XXX u64?
    type Hash: Debuggable + Clone + Eq = [u8; 32];
    type PublicKey: Debuggable + Clone + Eq = [u8; 64];
    type Signature: Debuggable + Clone + Eq = [u8; 65]; // XXX
    type EventId: Debuggable + Clone + Eq + Ord;
    type Event: Debuggable + Clone + Eq;

    fn zero_hash() -> Self::Hash;
    fn hash_bytes(data: &[u8]) -> Self::Hash;
    fn recover_address(data: &[u8], signature: Self::Signature) -> Result<Self::Address, Reason>;
    fn sign_message(message: &[u8]) -> Result<Self::Signature, Reason>;
    fn to_address(addr: &str) -> Result<Self::Address, Reason>;
    fn signer_address() -> Result<Self::Address, Reason>;
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Compound {}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Ethereum {}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Polkadot {}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Solana {}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Tezos {}

impl Chain for Compound {
    const ID: ChainId = ChainId::Comp;

    type EventId = comp::EventId;
    type Event = comp::Event;

    fn zero_hash() -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn hash_bytes(_data: &[u8]) -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn recover_address(_data: &[u8], _signature: Self::Signature) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn sign_message(_message: &[u8]) -> Result<Self::Signature, Reason> {
        panic!("XXX not implemented");
    }

    fn to_address(_addr: &str) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn signer_address() -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }
}

impl Chain for Ethereum {
    const ID: ChainId = ChainId::Eth;

    type EventId = eth::EventId;
    type Event = eth::Event;

    fn zero_hash() -> Self::Hash {
        [0u8; 32]
    }

    fn hash_bytes(data: &[u8]) -> Self::Hash {
        use tiny_keccak::Hasher;
        let mut hash = [0u8; 32];
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(&data[..]);
        hasher.finalize(&mut hash);
        hash
    }

    fn recover_address(data: &[u8], signature: Self::Signature) -> Result<Self::Address, Reason> {
        Ok(compound_crypto::eth_recover(data, &signature, true)?)
    }

    fn sign_message(message: &[u8]) -> Result<Self::Signature, Reason> {
        let message = Vec::from(message);
        let eth_key_id = runtime_interfaces::validator_config_interface::get_eth_key_id()
            .ok_or(Reason::KeyNotFound)?;
        Ok(runtime_interfaces::keyring_interface::sign_one(
            message, eth_key_id,
        )?)
    }

    fn to_address(addr: &str) -> Result<Self::Address, Reason> {
        if addr.len() == 42 && &addr[0..2] == "0x" {
            if let Ok(bytes) = hex::decode(&addr[2..42]) {
                if let Ok(address) = bytes.try_into() {
                    return Ok(address);
                }
            }
        }
        return Err(Reason::BadAddress);
    }

    fn signer_address() -> Result<Self::Address, Reason> {
        let eth_key_id = runtime_interfaces::validator_config_interface::get_eth_key_id()
            .ok_or(Reason::KeyNotFound)?;
        let pubk = runtime_interfaces::keyring_interface::get_public_key(eth_key_id)?;

        Ok(public_key_bytes_to_eth_address(&pubk))
    }
}

impl Chain for Polkadot {
    const ID: ChainId = ChainId::Dot;

    type EventId = dot::EventId;
    type Event = dot::Event;

    fn zero_hash() -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn hash_bytes(_data: &[u8]) -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn recover_address(_data: &[u8], _signature: Self::Signature) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn sign_message(_message: &[u8]) -> Result<Self::Signature, Reason> {
        panic!("XXX not implemented");
    }

    fn to_address(_addr: &str) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn signer_address() -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }
}

impl Chain for Solana {
    const ID: ChainId = ChainId::Sol;

    type EventId = sol::EventId;
    type Event = sol::Event;

    fn zero_hash() -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn hash_bytes(_data: &[u8]) -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn recover_address(_data: &[u8], _signature: Self::Signature) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn sign_message(_message: &[u8]) -> Result<Self::Signature, Reason> {
        panic!("XXX not implemented");
    }

    fn to_address(_addr: &str) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn signer_address() -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }
}

impl Chain for Tezos {
    const ID: ChainId = ChainId::Tez;

    type EventId = tez::EventId;
    type Event = tez::Event;

    fn zero_hash() -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn hash_bytes(_data: &[u8]) -> Self::Hash {
        panic!("XXX not implemented");
    }

    fn recover_address(_data: &[u8], _signature: Self::Signature) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn sign_message(_message: &[u8]) -> Result<Self::Signature, Reason> {
        panic!("XXX not implemented");
    }

    fn to_address(_addr: &str) -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }

    fn signer_address() -> Result<Self::Address, Reason> {
        panic!("XXX not implemented");
    }
}

// XXX technically all the remaining mod::types I think could become ADTs instead
//  which would also be a union type that would allow us to store them together
//  in general storing types which add variants for chains over time *must* be ok
//   or this strategy breaks and we need to re-visit everywhere in storage that's happening
pub mod comp {
    use codec::{Decode, Encode};
    use our_std::RuntimeDebug;

    pub type EventId = (u64, u64); // XXX

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct Event {}
}

pub mod eth {
    // Note: The substrate build requires these be imported
    pub use our_std::vec::Vec;

    use super::{Chain, Ethereum};
    use codec::{Decode, Encode};
    use our_std::RuntimeDebug;

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub enum RecoveryError {
        SignatureRecoveryError,
    }

    pub type BlockNumber = u32;
    pub type LogIndex = u32;

    pub type EventId = (BlockNumber, LogIndex);

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct Event {
        pub id: EventId,
        pub data: EventData,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub enum EventData {
        // XXX only event is 'do'?
        Lock {
            asset: <Ethereum as Chain>::Address,
            holder: <Ethereum as Chain>::Address,
            amount: <Ethereum as Chain>::Amount,
        },

        LockCash {
            holder: <Ethereum as Chain>::Address,
            amount: <Ethereum as Chain>::Amount,
            index: <Ethereum as Chain>::CashIndex,
        },

        Gov {
            // XXX all these become do?
        },
    }
}

pub mod dot {
    use codec::{Decode, Encode};
    use our_std::RuntimeDebug;

    pub type EventId = (u64, u64);

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct Event {}
}

pub mod sol {
    use codec::{Decode, Encode};
    use our_std::RuntimeDebug;

    pub type EventId = (u64, u64);

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct Event {}
}

pub mod tez {
    use codec::{Decode, Encode};
    use our_std::RuntimeDebug;

    pub type EventId = (u128, u128);

    #[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
    pub struct Event {}
}
