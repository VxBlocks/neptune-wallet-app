import { createSlice, createAsyncThunk } from "@reduxjs/toolkit";
import { Wallet, WalletState } from "../types";
import { getCurrentWallet, getWalletAddress, getWallets } from "@/commands/wallet";
import { requestWalletBalance } from "@/utils/api/apis";
import { WalletBalanceData } from "@/utils/api/types"; 

const initialState: WalletState = {
    currentAddress: "",
    currentWalledId: -1,
    wallets: [],
    loadingWallets: false,

    loadingBalance: false,
    balanceData: {
        available_balance: "0",
        total_balance: "0",
    },
    mnemonic: "",
    oneTimeWalletName: "",
    oneTimePassword: "" 
}

const walletSlice = createSlice({
    name: "wallet",
    initialState,
    reducers: {
        setMnemonic: (state, action) => {
            state.mnemonic = action.payload;
        },
        setOneTimeWalletName: (state, action) => {
            state.oneTimeWalletName = action.payload;
        },
        setOneTimePassword: (state, action) => {
            state.oneTimePassword = action.payload;
        }
    },
    extraReducers: (builder) => {
        builder.addCase(queryWallets.pending, (state, action) => {
            state.loadingWallets = true;
        });
        builder.addCase(queryWallets.rejected, (state, action) => {
            state.loadingWallets = false;
        });
        builder.addCase(queryWallets.fulfilled, (state, action) => {
            state.loadingWallets = false;
            state.wallets = action.payload.data;
            state.currentAddress = action.payload.currentAddress;
            state.currentWalledId = action.payload.walletId;
        });
         builder.addCase(queryCurrentWalletID.fulfilled, (state, action) => {
            state.currentWalledId = action.payload.data;
        }); 
        builder.addCase(queryCurrentWallet.fulfilled, (state, action) => {
            state.currentAddress = action.payload.currentAddress;
        });


        builder.addCase(queryWalletBalance.pending, (state, action) => {
            state.loadingBalance = true;
        });
        builder.addCase(queryWalletBalance.rejected, (state, action) => {
            state.loadingBalance = false;
        });
        builder.addCase(queryWalletBalance.fulfilled, (state, action) => {
            state.loadingBalance = false;
            state.balanceData = action.payload.data;
        });
    }
})


export const queryWallets = createAsyncThunk<
    { data: Wallet[], currentAddress: string, walletId: number }
>(
    '/api/wallet/queryWallets',
    async () => {
        const res = await getWallets(); 
        const currentAddress = await getWalletAddress(0);
        const walletId = await getCurrentWallet();
        return {
            data: res as Wallet[],
            currentAddress,
            walletId
        }
    }
) 
export const queryCurrentWalletID = createAsyncThunk<
    { data: number }
>(
    '/api/wallet/queryCurrentWalletID',
    async () => {
        const req = await getCurrentWallet();  
        return {
            data:req
        }
    }
)

export const queryWalletBalance = createAsyncThunk<
    { data: WalletBalanceData },
    { serverUrl: string }
>(
    '/api/balance/queryWalletBalance',
    async ({ serverUrl }) => {
        const req = await requestWalletBalance({ serverUrl });
        let balanceData = req.data
        return {
            data: {
                available_balance: amount_to_fixed(balanceData.available_balance),
                total_balance: amount_to_fixed(balanceData.total_balance)
            }
        }
    }
)

function amount_to_fixed(amount: string) {
    if (!amount) return "0"
    let len = amount.length;
    return amount.substring(0, len - 30)
}

export const queryCurrentWallet = createAsyncThunk<
    { currentAddress: string }
>(
    '/api/wallet/queryCurrentWallet',
    async () => {
        const currentAddress = await getWalletAddress(0);
        return {
            currentAddress
        }
    }
)

export const {
    setMnemonic,
    setOneTimeWalletName,
    setOneTimePassword
} = walletSlice.actions;

export default walletSlice.reducer;