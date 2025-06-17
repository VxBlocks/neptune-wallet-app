import { configureStore } from "@reduxjs/toolkit";

import walletSlice from "./wallet/wallet-slice";
import aboutSlice from "./about/about-slice";
import historySlice from "./history/history-slice";
import settingsSlice from "./settings/settings-slice";
import authSlice from "./auth/auth-slice";
import logSlice from "./log/log-slice";
import syncSlice from "./sync/sync-slice";
import executionSlice from "./execution/execution-slice";
import contactSlice from "./contact/contact-slice";

export const store = configureStore({
    reducer: {
        wallet: walletSlice,
        about: aboutSlice,
        history: historySlice,
        settings: settingsSlice,
        auth: authSlice,
        log: logSlice,
        sync: syncSlice,
        execution: executionSlice,
        contact: contactSlice
    }
});

export type RootState = ReturnType<typeof store.getState>;

export type AppDispatch = typeof store.dispatch;
