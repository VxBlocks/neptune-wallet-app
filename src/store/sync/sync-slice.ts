import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { SyncState } from "../types";
import { requestLatestBlock, requestStartScanBlockStatus } from "@/utils/api/apis";

const initialState: SyncState = {
    latestBlock: 0,
    syncing: false,
    syncedBlock: 0,
    syncPercentage: 0,
    syncingData: {
        height: 0,
        syncing: false,
        updated_to_tip: false
    }
}

const syncSlice = createSlice({
    name: "sync",
    initialState,
    reducers: {
        updateSyncedBlock: (state, action) => {
            state.syncedBlock = action.payload;
        },
        syncFinishBlock: (state, action) => {
            state.syncedBlock = action.payload;
            state.latestBlock = action.payload;
        }
    },
    extraReducers: (builder) => {
        builder.addCase(queryLatestBlock.fulfilled, (state, action) => {
            state.latestBlock = action.payload.data;
        });
        builder.addCase(querySyncBlockStatus.fulfilled, (state, action) => {
            state.syncingData = action.payload.data;
            state.syncedBlock = action.payload.syncedBlock;
        });
        builder.addCase(handleFinishBlockStatus.fulfilled, (state, action) => {
            state.syncedBlock = action.payload.data;
            state.latestBlock = action.payload.data;
        });
    }
})
export const queryLatestBlock = createAsyncThunk<
    { data: number },
    { serverUrl: string }
>(
    '/api/sync/queryLatestBlock',
    async ({ serverUrl }) => {
        const req = await requestLatestBlock({ serverUrl });
        let latestBlock = req.data;
        return {
            data: latestBlock
        }
    }
)


export const querySyncBlockStatus = createAsyncThunk<
    { data: any, syncedBlock: number },
    { serverUrl: string }
>(
    '/api/sync/querySyncBlockStatus',
    async ({ serverUrl }) => {
        const req = await requestStartScanBlockStatus({ serverUrl });
        let data = req.data;
        let syncedBlock = data.height;
        return {
            data,
            syncedBlock
        }
    }
)
export const handleFinishBlockStatus = createAsyncThunk<
    { data: any },
    { serverUrl: string }
>(
    '/api/sync/handleFinishBlockStatus',
    async ({ serverUrl }) => {
        const req = await requestLatestBlock({ serverUrl });
        let latestBlock = req.data;
        return {
            data: latestBlock
        }
    }
)

export const {
    updateSyncedBlock,
    syncFinishBlock
} = syncSlice.actions;

export default syncSlice.reducer;