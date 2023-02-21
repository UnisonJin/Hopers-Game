use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMIN: Item<Addr> = Item::new("owner");
pub const PRICE: Item<u128> = Item::new("price");
