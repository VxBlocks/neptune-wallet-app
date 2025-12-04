use anyhow::Result;
use neptune_privacy::api::export::Timestamp;
use neptune_privacy::protocol::consensus::type_scripts::native_currency_amount::NativeCurrencyAmount;
use serde::Deserialize;
use serde::Serialize;

impl super::WalletState {
    pub async fn get_balance(&self) -> Result<NativeCurrencyAmount> {
        let utxos = self.get_utxos().await?;

        let now = Timestamp::now();

        let mut balance = 0i128;
        for utxo in utxos {
            if utxo.spent_in_block.is_none() {
                if utxo.recovery_data.utxo.can_spend_at(now) {
                    let value = utxo.recovery_data.utxo.get_native_currency_amount();
                    balance += value.to_nau();
                }
            }
        }

        Ok(NativeCurrencyAmount::from_nau(balance))
    }

    pub async fn get_balance_history(&self) -> Result<Vec<WalletHistory>> {
        let utxos = self.get_utxos().await?;
        let mut history = Vec::new();
        for utxo in utxos {
            history.push(WalletHistory {
                amount: utxo
                    .recovery_data
                    .utxo
                    .get_native_currency_amount()
                    .display_lossless(),
                timestamp: utxo.confirmed_in_block.timestamp,
                height: utxo.confirmed_in_block.block_height,
                index: utxo.recovery_data.aocl_index,
                release_date: utxo.recovery_data.utxo.release_date(),
                txid: utxo.confirmed_txid,
            });
            if let Some(spent_in_block) = utxo.spent_in_block {
                history.push(WalletHistory {
                    amount: "-".to_string()
                        + &utxo
                            .recovery_data
                            .utxo
                            .get_native_currency_amount()
                            .display_lossless(),
                    timestamp: spent_in_block.timestamp,
                    height: spent_in_block.block_height,
                    index: utxo.recovery_data.aocl_index,
                    release_date: utxo.recovery_data.utxo.release_date(),
                    txid: utxo.spent_txid,
                })
            }
        }

        Ok(history)
    }

    pub async fn get_all_balance(&self) -> Result<(NativeCurrencyAmount, NativeCurrencyAmount)> {
        let utxos = self.get_utxos().await?;
        let now = Timestamp::now();

        let mut balance = 0i128;
        let mut locked = 0i128;
        for utxo in utxos {
            if utxo.spent_in_block.is_none() {
                let value = utxo.recovery_data.utxo.get_native_currency_amount();
                if utxo.recovery_data.utxo.can_spend_at(now) {
                    balance += value.to_nau();
                } else {
                    locked += value.to_nau();
                }
            }
        }

        Ok((
            NativeCurrencyAmount::from_nau(balance),
            NativeCurrencyAmount::from_nau(balance + locked),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletHistory {
    pub amount: String,
    pub timestamp: Timestamp,
    pub height: u64,
    pub index: u64,
    pub release_date: Option<Timestamp>,
    pub txid: Option<String>,
}
