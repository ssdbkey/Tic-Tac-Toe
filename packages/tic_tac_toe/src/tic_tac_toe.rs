use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    NewGame {},
    JoinGame {
        game_id: Uint128,
    },
    Play {
        game_id: Uint128,
        x_pos: u16,
        y_pos: u16,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GameInfoById {
        game_id: Uint128,
    },
    NewGames {
        start_after: Option<Uint128>,
        limit: Option<u32>,
    },
    PlayingGames {
        start_after: Option<Uint128>,
        limit: Option<u32>,
    },
    CompletedGames {
        start_after: Option<Uint128>,
        limit: Option<u32>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
