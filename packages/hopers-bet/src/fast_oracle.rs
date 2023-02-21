use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod msg {
    use cosmwasm_std::{Addr, Uint128};

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub struct InstantiateMsg {}

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ExecuteMsg {
        Update { price: Uint128 },
        Owner { owner: Addr },
    }

    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        Price {},
    }
}
