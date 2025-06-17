import { createSlice, createAsyncThunk } from "@reduxjs/toolkit";
import { AboutState, BuildInfo, UpdateVersion } from "../types";
import { getTauriVersion, getVersion } from '@tauri-apps/api/app';
import { get_build_info, get_update_info } from "@/commands/app";

const initialState: AboutState = {
    loadingAbout: false,
    buildInfo: null,
    version: "",
    tauriVersion: "",
    updateVersion: null
}

const aboutSlice = createSlice({
    name: "about",
    initialState,
    reducers: {

    },
    extraReducers: (builder) => {
        builder.addCase(queryAboutInfo.pending, (state, action) => {
            state.loadingAbout = true;
        });
        builder.addCase(queryAboutInfo.rejected, (state, action) => {
            state.loadingAbout = false;
        });
        builder.addCase(queryAboutInfo.fulfilled, (state, action) => {
            state.loadingAbout = false;
            state.buildInfo = action.payload.data;
            state.version = action.payload.version;
            state.tauriVersion = action.payload.tauriVersion;
        }); 
        builder.addCase(checkHasUpdateVersion.fulfilled, (state, action) => {
            state.updateVersion = action.payload.data
        });
    }
})


export const queryAboutInfo = createAsyncThunk<
    { data: BuildInfo, version: string, tauriVersion: string }
>(
    '/api/about/queryAboutInfo',
    async () => {
        const buildInfo = await get_build_info();
        let tauriVersion = await getTauriVersion();
        let version = await getVersion();
        return {
            data: buildInfo,
            version,
            tauriVersion
        }
    }
)

export const checkHasUpdateVersion = createAsyncThunk<
    { data: UpdateVersion }
>(
    '/api/about/checkHasUpdateVersion',
    async () => {
        const req = await get_update_info();
        return {
            data: req
        }
    }
)


export const {
} = aboutSlice.actions;

export default aboutSlice.reducer;