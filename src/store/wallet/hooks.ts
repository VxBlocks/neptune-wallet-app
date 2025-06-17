import { useAppSelector } from "@/store/hooks";

export const useMnemonic = () => {
    return useAppSelector(state => state.wallet.mnemonic);
} 
export const useOneTimeWalletName = () => {
    return useAppSelector(state => state.wallet.oneTimeWalletName);
} 
export const useOneTimePassword = () => {
    return useAppSelector(state => state.wallet.oneTimePassword);
} 
export const useLoadingWallets = () => {
    return useAppSelector(state => state.wallet.loadingWallets);
}
export const useWallets = () => {
    return useAppSelector(state => state.wallet.wallets);
} 
export const useCurrentAddress = () => { 
    return useAppSelector(state => state.wallet.currentAddress);
}
export const useCurrentWalledId = () => { 
    return useAppSelector(state => state.wallet.currentWalledId);
}
export const useLoadingBalance = () => { 
    return useAppSelector(state => state.wallet.loadingBalance);
}   
export const useBalanceData = () => { 
    return useAppSelector(state => state.wallet.balanceData);
}