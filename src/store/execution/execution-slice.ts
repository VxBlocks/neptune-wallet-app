import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { ExecutionState } from "../types";
import { ExecutionHistory } from "@/database/types/localhistory";
import { requestPendingTransactions, sendTransactionRequest } from "@/utils/api/apis";
import { PendingTransaction, SendInputItem, SendTransactionParam, SendTransactionResponse } from "@/utils/api/types";
import { addExecutionHistory, deleteExecutionHistory, getExecutionHistory } from "@/utils/storage"; 
const initialState: ExecutionState = {
    loadingExecution: false,
    executionData: [],
    send_state: "",
    executionPending: false,
    requesetSendTransactionResponse: {
        transaction: null,
        message: ""
    }
}

const executionSlice = createSlice({
    name: "execution",
    initialState,
    reducers: {
        updateSendState: (state, action) => {
            state.send_state = action.payload;
        }
    },
    extraReducers: (builder) => {

        builder.addCase(addExecutionTransactionHistory.fulfilled, (state, action) => {
            state.executionData = action.payload.data;
        });

        builder.addCase(queryExecutionHistorys.pending, (state) => {
            state.loadingExecution = true;
        });
        builder.addCase(queryExecutionHistorys.rejected, (state) => {
            state.loadingExecution = false;
        });
        builder.addCase(queryExecutionHistorys.fulfilled, (state, action) => {
            state.loadingExecution = false;
            state.executionData = action.payload.data;
        });
        builder.addCase(removeExecutionTransactionHistory.fulfilled, (state, action) => {
            state.executionData = action.payload.data;
        });
        builder.addCase(requestSedExecutionTransaction.pending, (state) => {
            state.executionPending = true;
        });
        builder.addCase(requestSedExecutionTransaction.rejected, (state) => {
            state.executionPending = false;
        });
        builder.addCase(requestSedExecutionTransaction.fulfilled, (state, action) => {
            state.requesetSendTransactionResponse = action.payload.data;
            state.executionData = action.payload.newLocalHistory;
            state.send_state = "";
            state.executionPending = false;
        });
    }
})

export const queryExecutionHistorys = createAsyncThunk<
    { data: ExecutionHistory[] },
    { addressId: number, serverUrl: string }
>(
    '/api/execution/queryExecutionHistorys',
    async ({ addressId, serverUrl }) => {
        const res = await queryPendingTransactions(addressId, serverUrl);
        return {
            data: res
        }
    }
)

async function queryPendingTransactions(addressId: number, serverUrl: string) {
    let newLocalHistory = [] as ExecutionHistory[];
    let localHistory = [] as ExecutionHistory[];
    let pendingHistory = [] as PendingTransaction[];
    try {
        localHistory = await getExecutionHistory({ addressId });
        let req = await requestPendingTransactions({ serverUrl });
        pendingHistory = req.data;
        if (pendingHistory && pendingHistory.length > 0) {
            if (localHistory && localHistory.length > 0) {
                pendingHistory.forEach((item) => {
                    let tx_id = item.tx_id;
                    let findLocalHistory = localHistory.find((history) => history.txid === tx_id);
                    if (findLocalHistory) {
                        newLocalHistory.push({
                            ...findLocalHistory,
                            outputs: findLocalHistory.status ? JSON.parse(findLocalHistory.status) : [],
                            status: item.status
                        });
                    }
                })
            } else {
                pendingHistory.forEach((item) => {
                    let tx_id = item.tx_id;
                    let status = item.status;
                    newLocalHistory.push({
                        txid: tx_id,
                        timestamp: 0,
                        height: 0,
                        addressId: addressId,
                        address: "",
                        fee: "",
                        priorityFee: "",
                        status,
                        outputs: [],
                        batchOutput: []
                    });
                })
            }
        }
    } catch (error: any) {
        console.log(error);

    }
    return newLocalHistory;
}

export const addExecutionTransactionHistory = createAsyncThunk<
    { data: ExecutionHistory[] },
    { history: ExecutionHistory, serverUrl: string, addressId: number }
>(
    '/api/execution/addExecutionTransactionHistory',
    async ({ addressId, history, serverUrl }) => {
        let newLocalHistory = [] as ExecutionHistory[];
        try {
            await addExecutionHistory({ localHistory: history })
            newLocalHistory = await queryPendingTransactions(addressId, serverUrl);
        } catch (error) {
        }
        return {
            data: newLocalHistory
        }
    }
)

export const removeExecutionTransactionHistory = createAsyncThunk<
    { data: ExecutionHistory[] },
    { txid: string, addressId: number, serverUrl: string }
>(
    '/api/execution/removeExecutionTransactionHistory',
    async ({ txid, addressId, serverUrl }) => {
        await deleteExecutionHistory({ txid })
        const res = await queryPendingTransactions(addressId, serverUrl);
        return {
            data: res
        }
    }
)

export const requestSedExecutionTransaction = createAsyncThunk<
    { data: any, newLocalHistory: ExecutionHistory[] },
    { serverUrl: string, param: SendTransactionParam, sendInputs: SendInputItem[], syncedBlock: number, currentWalletID: number, currentAddress: string }
>(
    '/api/execution/requestSedExecutionTransaction',
    async ({ serverUrl, param, sendInputs, currentWalletID, currentAddress, syncedBlock }) => {
        let transaction = null
        let message = ""
        let newLocalHistory = [] as ExecutionHistory[];
        try {
            const rep = await sendTransactionRequest({ serverUrl, param });
            if (rep.data) {
                transaction = rep.data as SendTransactionResponse
                let history = {
                    txid: transaction.txid,
                    timestamp: new Date().getTime(),
                    height: syncedBlock,
                    addressId: currentWalletID,
                    address: currentAddress,
                    fee: param.fee,
                    priorityFee: "",
                    status: "",
                    outputs: transaction.outputs,
                    batchOutput: sendInputs
                } as ExecutionHistory
                try {
                    await addExecutionHistory({ localHistory: history })
                    newLocalHistory = await queryPendingTransactions(currentWalletID, serverUrl);
                } catch (error) {
                }
            } else {
                message = "Send transaction failed!"
            }
        } catch (error: any) {
            let errorMessage = "Send transaction failed!"
            if (error.response && error.response.data) {
                errorMessage = error.response.data
            }
            message = errorMessage
        }

        return {
            data: {
                transaction,
                message: message
            },
            newLocalHistory
        }
    }
)

export const {
    updateSendState
} = executionSlice.actions;

export default executionSlice.reducer;