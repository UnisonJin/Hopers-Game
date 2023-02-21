use crate::{Config, FinishedRound, LiveRound, NextRound};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use hopers_bet::price_prediction::Direction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const IS_HAULTED: Item<bool> = Item::new("is_haulted");
pub const CONFIG: Item<Config> = Item::new("config");
pub const NEXT_ROUND_ID: Item<u128> = Item::new("next_round_id");
/* The round that's open for betting */
pub const NEXT_ROUND: Item<NextRound> = Item::new("next_round");
/* The live round; not accepting bets */
pub const LIVE_ROUND: Item<LiveRound> = Item::new("live_round");

pub const ACCUMULATED_FEE: Item<u128> = Item::new("accumulated_fee");

pub const ROUNDS: Map<u128, FinishedRound> = Map::new("rounds");

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct BetInfo {
    pub player: Addr,
    pub round_id: Uint128,
    pub amount: Uint128,
    pub direction: Direction,
}

/// Primary key for betinfo: (round_id, player)
pub type BetInfoKey = (u128, Addr);
/// Convenience bid key constructor
pub fn bet_info_key(round_id: u128, player: &Addr) -> BetInfoKey {
    (round_id, player.clone())
}

/// Defines incides for accessing bids
pub struct BetInfoIndicies<'a> {
    pub player: MultiIndex<'a, Addr, BetInfo, BetInfoKey>,
}

impl<'a> IndexList<BetInfo> for BetInfoIndicies<'a> {
    fn get_indexes(
        &'_ self,
    ) -> Box<dyn Iterator<Item = &'_ dyn Index<BetInfo>> + '_> {
        let v: Vec<&dyn Index<BetInfo>> = vec![&self.player];
        Box::new(v.into_iter())
    }
}

pub fn bet_info_storage<'a>(
) -> IndexedMap<'a, BetInfoKey, BetInfo, BetInfoIndicies<'a>> {
    let indexes = BetInfoIndicies {
        player: MultiIndex::new(
            |_pk: &[u8], d: &BetInfo| d.player.clone(),
            "bet_info",
            "bet_info_collection",
        ),
    };
    IndexedMap::new("bet_info", indexes)
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MyGameResponse {
    pub my_game_list: Vec<BetInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PendingRewardResponse {
    pub pending_reward: Uint128,
}
