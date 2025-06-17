import {
    LATEST_BLOCKHEIGHT,
    SCAN_BLOCK_STATE,
    WALLET_ACTIVITY_HISTORY,
    WALLET_AVAILABLE_UTXOS,
    WALLET_BALANCE,
    WALLET_FORGET_TX,
    WALLET_PENDING_HISTORY,
    WALLET_SEND_TRANSACTION
} from "@/constant";
import service, {url} from "@/utils/api/service";
import {handleServiceUrl} from "../url";
import {SendTransactionParam} from "./types";


export const requestLatestBlock = ({
                                       serverUrl,
                                   }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${LATEST_BLOCKHEIGHT}`),
        method: "GET",
    });
}

export const requestStartScanBlockStatus = ({
                                                serverUrl,
                                            }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${SCAN_BLOCK_STATE}`),
        method: "GET",
    });
}

export const requestWalletBalance = ({
                                         serverUrl,
                                     }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_BALANCE}`),
        method: "GET",
    });
}


export const requestActivityTransactions = ({
                                                serverUrl,
                                            }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_ACTIVITY_HISTORY}`),
        method: "GET",
    });
}

export const requestPendingTransactions = ({
                                               serverUrl,
                                           }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_PENDING_HISTORY}`),
        method: "GET",
    });
}

export const forgetPendingTransaction = ({
                                             serverUrl,
                                             txid
                                         }: {
    serverUrl: string,
    txid: string
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_FORGET_TX}${txid}`),
        method: "GET",
    });
}

export const sendTransactionRequest = ({
                                           serverUrl,
                                           param
                                       }: {
    serverUrl: string,
    param: SendTransactionParam
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_SEND_TRANSACTION}`),
        method: "POST",
        data: {...param},
    });
}

export const requestAvailableUtxos = ({
                                          serverUrl,
                                      }: {
    serverUrl: string,
}) => {
    let {rpc} = handleServiceUrl(serverUrl)
    return service({
        url: url(`${rpc}${WALLET_AVAILABLE_UTXOS}`),
        method: "GET",
    });
}