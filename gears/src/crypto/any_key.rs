use keyring::key::pair::KeyPair;

use super::keys::{GearsPublicKey, ReadAccAddress, SigningKey};
#[cfg(feature = "ledger")]
use super::ledger::{LedgerError, LedgerProxyKey};
#[cfg(not(feature = "ledger"))]
type LedgerError = core::convert::Infallible;

#[derive(Debug)]
pub enum AnyKey {
    Local(KeyPair),
    #[cfg(feature = "ledger")]
    Ledger(LedgerProxyKey),
}

impl ReadAccAddress for AnyKey {
    fn get_address(&self) -> address::AccAddress {
        match self {
            AnyKey::Local(k) => k.get_address(),
            #[cfg(feature = "ledger")]
            AnyKey::Ledger(k) => k.get_address(),
        }
    }
}

impl GearsPublicKey for AnyKey {
    fn get_gears_public_key(&self) -> super::public::PublicKey {
        match self {
            AnyKey::Local(k) => k.get_gears_public_key(),
            #[cfg(feature = "ledger")]
            AnyKey::Ledger(k) => k.get_gears_public_key(),
        }
    }
}

impl SigningKey for AnyKey {
    type Error = LedgerError;

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Self::Error> {
        match self {
            AnyKey::Local(k) => Ok(k.sign(message)),
            #[cfg(feature = "ledger")]
            AnyKey::Ledger(k) => k.sign(message),
        }
    }
}
