export interface SendTransactionParam {
    outputs: Output[]
    fee: string,
    inputs: number[]
}

export interface SendTransactionResponse {
    txid: string
    outputs: string[]
}

export interface Output {
    address: string;
    amount: string
}

export interface WalletBalanceData {
    available_balance: string;
    total_balance: string;
}


export interface PendingTransaction {
    tx_id: string;
    status: string;
}

export interface HistoryData {
    amount: string,
    timestamp: number,
    height: number,
    index: number,
    release_date: any,
    txid: string
}


export interface SendInputItem {
    index: number,
    toAddress: string,
    amount: string,
}

