import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { HistoryState, HistoryUtxo, MerageHistory, UtxoItem } from "../types";
import { requestActivityTransactions, requestAvailableUtxos } from "@/utils/api/apis";
import { ExecutionHistory } from "@/database/types/localhistory";
import { HistoryData } from "@/utils/api/types";
import { amount_to_positive_fixed } from "@/utils/math-util";
import { bigNumberPlusToString } from "@/utils/common";
import { getExecutionHistory } from "@/utils/storage";

const initialState: HistoryState = {
    loadingActivityHistory: false,
    activityHistory: [],
    inExecutionTx: null,
    loadingAvailableUtxos: false,
    availableUtxos: []
}

const historySlice = createSlice({
    name: "history",
    initialState,
    reducers: {},
    extraReducers: (builder) => {
        builder.addCase(queryActivityHistory.pending, (state) => {
            state.loadingActivityHistory = true;
        });
        builder.addCase(queryActivityHistory.rejected, (state) => {
            state.loadingActivityHistory = false;
        });
        builder.addCase(queryActivityHistory.fulfilled, (state, action) => {
            state.loadingActivityHistory = false;
            state.activityHistory = action.payload.data;
        });

        builder.addCase(queryAvailableUtxosList.pending, (state) => {
            state.loadingAvailableUtxos = true;
        });
        builder.addCase(queryAvailableUtxosList.rejected, (state) => {
            state.loadingAvailableUtxos = false;
        });
        builder.addCase(queryAvailableUtxosList.fulfilled, (state, action) => {
            state.loadingAvailableUtxos = false;
            state.availableUtxos = action.payload.data;
        });
    }
})
export const queryAvailableUtxosList = createAsyncThunk<
    { data: UtxoItem[] },
    { serverUrl: string, sortType?: string, containLocked?: boolean }
>(
    '/api/history/queryAvailableUtxosList',
    async ({ serverUrl, sortType = "Amount", containLocked = false }) => {
        let newList = [] as UtxoItem[];
        const res = await requestAvailableUtxos({ serverUrl });
        const data = res.data as UtxoItem[];
        if (sortType === "Amount") {
            data.sort((a, b) => Number(b.amount) - Number(a.amount))
        } else if (sortType === "ID") {
            data.sort((a, b) => Number(b.id) - Number(a.id))
        }
        newList = data
        if (!containLocked) {
            newList = data.filter(item => !item.locked)
        }
        return {
            data: newList
        }
    }
)


export const queryActivityHistory = createAsyncThunk<
    { data: MerageHistory[] },
    { serverUrl: string, addressId: number, historyType?: string }
>(
    '/api/history/queryActivityHistory',
    async ({ serverUrl, addressId, historyType = "All" }) => {
        const res = await requestActivityTransactions({ serverUrl });
        const data = res.data as HistoryData[];
        let merageHistorys = await handleTransaction(data, addressId, historyType);
        if (merageHistorys && merageHistorys.length > 0) {
            merageHistorys.sort((a, b) => b.timestamp - a.timestamp);
        }
        return {
            data: merageHistorys
        }
    }
)

async function handleTransaction(data: HistoryData[], addressId: number, historyType: string) {
    let merageHistorys = [] as MerageHistory[];
    let localHistory: ExecutionHistory[] = [];
    let queryHistory = data;
    try {
        localHistory = await getExecutionHistory({ addressId });
    } catch (error) {

    }
    let newQueryHistory = [] as HistoryData[];
    queryHistory && queryHistory.length > 0 && queryHistory.forEach(element => {
        let hasHeight = newQueryHistory.find((item) => item.height === element.height)
        if (hasHeight) {
            let newAmount = bigNumberPlusToString(hasHeight.amount, element.amount)
            hasHeight.amount = newAmount;
            if (element.txid) {
                hasHeight.txid = element.txid
            }
        } else {
            newQueryHistory.push(element);
        }
    });
    newQueryHistory && newQueryHistory.forEach(element => {
        let hasTxId = element.txid && element.txid != "";
        let history = {} as MerageHistory;
        if (hasTxId) {
            let findHistoryByID = localHistory && localHistory.length > 0 && localHistory.find(item => item.txid === element.txid);
            if (findHistoryByID) {
                history.fee = findHistoryByID.fee;
                history.priorityFee = findHistoryByID.priorityFee;
                history.form = findHistoryByID.address
                history.outputs = findHistoryByID.outputs;
                history.batchOutput = findHistoryByID.batchOutput
            }
            history.txid = element.txid;
        }
        let amount = amount_to_positive_fixed(element.amount)
        let isPositive = !(element.amount && element.amount.startsWith("-"));
        if (isPositive) {
            history.message = "Received " + amount;
        } else {
            history.message = "Sent " + amount;
        }
        history.changeAmount = isPositive ? "+ " + amount : "- " + amount;
        history.timestamp = element.timestamp;
        history.height = element.height;
        history.index = element.index;
        history.release_date = element.release_date;
        history.utxos = findUtxoHistoryByHeight(data, element.height);
        if (historyType == "Send") {
            if (!isPositive) {
                merageHistorys.push(history);
            }
        } else if (historyType == "Receive") {
            if (isPositive) {
                merageHistorys.push(history);
            }
        } else {
            merageHistorys.push(history);
        } 
    });
    return merageHistorys;
}

function findUtxoHistoryByHeight(data: HistoryData[], height: number) {
    let utxoHistorys = [] as HistoryUtxo[];
    let newList = data.filter(item => item.height === height)
    newList && newList.forEach(element => {
        utxoHistorys.push({
            id: element.index,
            amount: element.amount
        })
    })
    return utxoHistorys;
}


export const { } = historySlice.actions;

export default historySlice.reducer;