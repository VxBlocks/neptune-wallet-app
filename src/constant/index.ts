// Latest height
export const LATEST_BLOCKHEIGHT = '/rpc/block/tip_height'

export const SCAN_BLOCK_STATE = '/rpc/scan/state'

export const START_SCAN_BLOCK = '/rpc/scan/0/0'
// History records
export const WALLET_ACTIVITY_HISTORY = '/rpc/wallet/history'

export const WALLET_AVAILABLE_UTXOS = '/rpc/wallet/available_utxos'

// Transaction history in progress
export const WALLET_PENDING_HISTORY = '/rpc/mempool/pendingtx'
// Cancel transaction /rpc/forget_tx/${txid}
export const WALLET_FORGET_TX = '/rpc/forget_tx/'
// Send transaction
export const WALLET_SEND_TRANSACTION = '/rpc/send'

// Wallet balance
export const WALLET_BALANCE = '/rpc/wallet/balance'

export const SYNC_HEIGHT_EVENT = "sync_height" // Sync progress
export const SYNC_STOP_EVENT = "sync_stop" // Stop syncing
export const SYNC_FINISH_EVENT = "sync_finish" // Sync completed
export const SYNC_NEW_BLOCK_EVENT = "syncing_new_block" // A new block height is received

export const SYNC_SENT_STATUS_EVENT = "send_state" // Status when sending a transaction

export const LOG_LEVELS = [
    { value: 'error', label: 'Error' },
    { value: 'warn', label: 'Warn' },
    { value: 'info', label: 'Info' },
    { value: 'debug', label: 'Debug' },
    { value: 'trace', label: 'Trace' }
]

export const NETWORKS = [
    { value: 'main', label: 'Mainnet' },
]