use std::fmt::{self, Display};

use anyhow::Result;
use derive_more::{Deref, DerefMut};
use octez::OctezClient;
use tezos_crypto_rs::hash::ContractKt1Hash;

use crate::BootstrapAccount;

const CTEZ_CONTRACT: &str = include_str!("../../../contracts/jstz_ctez.tz");
const BRIDGE_CONTRACT: &str = include_str!("../../../contracts/jstz_bridge.tz");

impl BootstrapAccount {
    fn as_michelson_elt(&self) -> String {
        format!("Elt \"{}\" {}", self.address, self.amount)
    }
}

pub fn deploy_ctez_contract<'a, I>(
    client: &OctezClient,
    operator_address: &str,
    bootstrap_accounts: I,
) -> Result<String>
where
    I: Iterator<Item = &'a BootstrapAccount>,
{
    let init_storage = format!(
        "(Pair \"{}\" {{ {} }} )",
        operator_address,
        bootstrap_accounts
            .map(BootstrapAccount::as_michelson_elt)
            .collect::<Vec<_>>()
            .join(" ")
    );

    client.originate_contract("jstz_ctez", operator_address, CTEZ_CONTRACT, &init_storage)
}

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct BridgeContract(String);

impl Display for BridgeContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ContractKt1Hash> for BridgeContract {
    fn from(hash: ContractKt1Hash) -> Self {
        Self(hash.to_base58_check())
    }
}

impl BridgeContract {
    pub fn deploy(
        client: &OctezClient,
        operator: &str,
        ctez_address: &str,
    ) -> Result<Self> {
        let init_storage = format!("(Pair \"{}\" None)", ctez_address);

        let bridge_address = client.originate_contract(
            "jstz_bridge",
            operator,
            BRIDGE_CONTRACT,
            &init_storage,
        )?;

        Ok(Self(bridge_address))
    }

    pub fn set_rollup(
        &self,
        client: &OctezClient,
        operator_address: &str,
        rollup_address: &str,
    ) -> Result<()> {
        client.call_contract(
            operator_address,
            &self.0,
            "set_rollup",
            &format!("\"{}\"", rollup_address),
        )
    }
}
