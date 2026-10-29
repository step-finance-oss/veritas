#![cfg_attr(feature = "swaps-only", allow(unused_imports))]
#![cfg_attr(feature = "swaps-only", allow(unused_variables))]
#![cfg_attr(feature = "swaps-only", allow(dead_code))]
use std::collections::HashMap;

use anyhow::Result;
use clickhouse::{Client, Row};
use rust_decimal::Decimal;
use serde::Deserialize;

pub type TokenBalanceCache = HashMap<String, Option<Decimal>>;

#[derive(Deserialize, Row)]
pub struct PoolVaultBalanceRow {
    pub vault_pubkey: String,
    pub balance: u64,
}

pub async fn build_token_balance_cache(
    client: &Client,
    skip_preloads: bool,
) -> Result<TokenBalanceCache> {
    #[cfg(feature = "swaps-only")]
    {
        log::info!("Skipping token balance cache build (swaps-only feature enabled)");
        Ok(TokenBalanceCache::new())
    }

    #[cfg(not(feature = "swaps-only"))]
    {
        if skip_preloads {
            return Ok(TokenBalanceCache::new());
        }
        log::info!("Building token balance cache...");

        let query = "
            SELECT
                base58Encode(vault_pubkey) AS vault_pubkey,
                balance
            FROM current_lp_vault_balance FINAL
            WHERE curve_type = 4
            OR curve_type = 7
        ";

        let mut cursor = client.query(query).fetch::<PoolVaultBalanceRow>()?;
        let mut cache = HashMap::new();
        let mut count = 0;

        while let Some(row) = cursor.next().await? {
            cache.insert(row.vault_pubkey, Some(row.balance.into()));
            count += 1;
        }

        log::info!("Token balance cache built with {} accounts", count);

        Ok(cache)
    }
}
