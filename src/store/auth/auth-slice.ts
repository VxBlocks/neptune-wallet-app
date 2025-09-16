import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { AuthData, AuthState } from "../types";
import { run_rpc_server, stop_rpc_server } from "@/commands/app";
import { has_password, try_password } from "@/commands/password";

const initialState: AuthState = {
    startRpcServer: false,
    data: {
        loading: false,
        hasPassword: false,
        hasAuth: false,
    } as AuthData,
}

const authSlice = createSlice({
    name: "auth",
    initialState,
    reducers: {

    },
    extraReducers: (builder) => {
        builder.addCase(checkAuthPassword.pending, (state, action) => {
            state.data = {
                ...state.data,
                loading: true,
            };
        });
        builder.addCase(checkAuthPassword.rejected, (state, action) => {
            state.data = {
                ...state.data,
                loading: false,
            };
        });
        builder.addCase(checkAuthPassword.fulfilled, (state, action) => {
            state.data = {
                hasAuth: action.payload.hasAuth,
                hasPassword: action.payload.hasPassword,
                loading: false,
            };
        });
        builder.addCase(startRunRpcServer.fulfilled, (state, action) => {
            state.startRpcServer = action.payload.data;
        });
    }
})

export const checkAuthPassword = createAsyncThunk<
    { hasPassword: boolean, hasAuth: boolean }
>(
    '/api/auth/checkAuthPassword',
    async () => {
        let hasPassword = false;
        let hasAuth = false;
        try {
            hasPassword = await has_password();
            if (hasPassword) {
                hasAuth = await try_password();
            }

        } catch (error) {
            console.log(error);
        }
        return { hasPassword, hasAuth };
    }
)


export const startRunRpcServer = createAsyncThunk<
    { data: boolean }
>(
    '/api/auth/startRunRpcServer',
    async () => {
        let startRpcServer = false;
        try {
            await run_rpc_server();
            startRpcServer = true;
        } catch (error) {
            console.log(error);
            if (error == "error start rpc: rpc server is already running") {
                startRpcServer = true
            }
        }

        return { data: startRpcServer };
    }
)
export const {
} = authSlice.actions;

export default authSlice.reducer;
