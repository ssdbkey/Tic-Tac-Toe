use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GameInfo {
    pub hoster: Addr,
    pub joiner: Addr,
    pub turn: u16, // 0: none, 1: hoster, 2: joiner
    pub board: [[u16; 3]; 3],
    pub winner: u16, // 0: none, 1: hoster, 2: joiner, 3: drawn
}

pub const LAST_GAME_ID: Item<u128> = Item::new("last_game_id");

// key = game_id
pub const GAME_LIST: Map<u128, GameInfo> = Map::new("game_list");
pub const NEW_GAME_LIST: Map<u128, bool> = Map::new("new_game_list");
pub const PLAYING_GAME_LIST: Map<u128, bool> = Map::new("playing_game_list");
pub const COMPLETED_GAME_LIST: Map<u128, bool> = Map::new("completed_game_list");
