//! This contract implements simple counter backed by storage on blockchain.
//!
//! The contract provides methods to [increment] / [decrement] counter and
//! get it's current value [get_num] or [reset].
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PromiseError, PanicOnDefault, Timestamp};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{U64, U128};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Counter {
    val: i8,
    pub usn_contract: AccountId,
}

use near_sdk::{ext_contract};

pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

pub type DurationSec = u32;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Clone, Copy, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub multiplier: U128,
    pub decimals: u8,
}

// From https://github.com/NearDeFi/price-oracle/blob/main/src/asset.rs
#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AssetOptionalPrice {
    pub asset_id: AccountId,
    pub price: Option<Price>,
}

// From https://github.com/NearDeFi/price-oracle/blob/main/src/lib.rs
#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PriceData {
    timestamp: U64,
    recency_duration_sec: DurationSec,
    prices: Vec<AssetOptionalPrice>,
}

// Interface of this contract, for callbacks
#[ext_contract(this_contract)]
trait Callbacks {
  fn query_price_callback(&mut self) -> bool;
}

// Validator interface, for cross-contract calls
#[ext_contract(usn_contract)]
trait PriceOracle {
  fn get_price_data(&self, asset_ids: Option<Vec<AccountId>>) -> PriceData;
}

impl Default  for PriceFetcher {
    fn default() -> Self {
       Self {
            val: 8,
            usn_contract: "priceoracle.testnet".parse().unwrap(),
       }
    }
}

#[near_bindgen]
impl PriceFetcher {

     #[init]
      #[private] // Public - but only callable by env::current_account_id()
      pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
        val: 8,
          usn_contract: "priceoracle.testnet".parse().unwrap(),
        }
      }

      // Public - query external greeting
      pub fn query_price(&self) -> Promise {
       let mut assets: Vec<AccountId> = Vec::new();
       assets.push("usdn.testnet".parse().unwrap());

        // Create a promise to call HelloNEAR.get_greeting()
        let promise = usn_contract::ext(self.usn_contract.clone())
          .with_static_gas(Gas(5*TGAS))
          .get_price_data(Some(assets));

        return promise.then( // Create a promise to callback query_greeting_callback
          Self::ext(env::current_account_id())
          .with_static_gas(Gas(5*TGAS))
          .query_price_callback()
        )
      }

      #[private] // Public - but only callable by env::current_account_id()
      pub fn query_price_callback(&self, #[callback_result] call_result: Result<PriceData, PromiseError>) -> PriceData {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
          log!("There was an error contacting priceoracle");
        }

        // Return the greeting
        let data: PriceData = call_result.unwrap();
        data
      }
}
