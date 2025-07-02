import { Contact } from "@/database/types/contact";
import { ExecutionHistory } from "@/database/types/localhistory";
import { SendInputItem, SendTransactionResponse, WalletBalanceData } from "@/utils/api/types";

export interface WalletState {
    mnemonic: string;
    oneTimeWalletName: string;
    oneTimePassword: string;
    currentAddress: string;
    currentWalledId: number;
    loadingWallets: boolean;
    wallets: Wallet[];

    loadingBalance: boolean;
    balanceData: WalletBalanceData

}

export interface Wallet {
    id: number,
    name: string,
    address: string,
    balance: string,
}

export interface AboutState {
    loadingAbout: boolean;
    buildInfo: BuildInfo | null;
    version: string;
    tauriVersion: string
    updateVersion: UpdateVersion | null
}

export interface UpdateVersion {
    version: string,
    url: string
}

export interface BuildInfo {
    time: string,
    commit: string,
}

export interface HistoryState {
    loadingActivityHistory: boolean;
    activityHistory: MerageHistory[];
    perDay: DayHistory[];
    loadingAvailableUtxos: boolean;
    availableUtxos: UtxoItem[];
    inExecutionTx: Transaction | null;
}

export interface Transaction {
    status: string,
    id: string,
    to: string,
    value: string,
    fee: string,
    priorityFee: string,
    proofType: string,
    timestamp: number,
}

export interface UtxoItem {
    id: string,
    hash: string,
    confirm_timestamp: number,
    confirm_height: number,
    confirmed_txid: string,
    locked: boolean,
    amount: string,
}

export interface Activity {
    id: string,
    from: string,
    to: string,
    fee: string,
    priorityFee: string,
    amount: string;
    timestamp: number;
    height: number;
    index: number;
    release_date: any;
}

export interface MerageHistory {
    txid: string,
    form: string,
    message: string, 
    changeAmount: string,
    fee: string,
    priorityFee: string,
    timestamp: number,
    index: number,
    height: number,
    release_date: any,
    outputs: string[],
    batchOutput?: SendInputItem[],
    utxos: HistoryUtxo[]
}

export interface DayHistory {
    start_height: number,
    end_height: number,
    Received: number,
    Spent: number,
    timestamp: number,
    data: string,
}


export interface HistoryUtxo {
    id: number,
    amount: string,
}


export interface Pending {
    id: string,
    value: string,
}

export interface SettingsState {
    loadingSettings: boolean,
    acctionData: SettingActionData,
    platform: string,
    cacheFiles: BlockCacheFile[],
}

export interface BlockCacheFile {
    path: string,
    network: string,
    range: number[],
}

export interface SettingActionData {
    serverUrl: string,
    network: string,
    logLevel: string,
    remoteUrl: string,
    password: string,
    system: Info,
}

export interface Info {
    os_type: string
    version: any
    edition?: string
    codename?: string
    bitness: string
    architecture?: string
}

export interface AuthState {
    startRpcServer: boolean,
    data: AuthData,
}

export interface AuthData {
    loading: boolean,
    hasAuth: boolean,
    hasPassword: boolean,
}

export interface LogState {
    loadingLogs: boolean,
    logs: string[],
}

export interface SyncState {
    latestBlock: number, 
    syncing: boolean, 
    syncedBlock: number, 
    syncPercentage: number, 
    syncingData: SyncingData
}

export interface SyncingData {
    height: number,
    syncing: boolean,
    updated_to_tip: boolean
}

export interface ExecutionState {
    loadingExecution: boolean,
    executionData: ExecutionHistory[],
    send_state: string, 
    executionPending: boolean, 
    requesetSendTransactionResponse: {
        transaction: SendTransactionResponse | null,
        message: string
    }
}

export interface ContactState {
    loadingContacts: boolean,
    contacts: Contact[],

}