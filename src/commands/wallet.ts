import { invoke } from "@tauri-apps/api/core";

export interface WalletData {
    id: number,
    name: string,
    address: string,
    balance: string,
}
export async function addWallet(name: String, mnemonic: String, num_keys: number, start_height: number, is_new: boolean): Promise<number> {
    return await invoke('add_wallet', { name: name, mnemonic: mnemonic, numKeys: num_keys, startHeight: start_height, isNew: is_new })
}
export async function setCurrentWallet(id: number) {
    await invoke('set_wallet_id', { id })
}
export async function getCurrentWallet(): Promise<number> {
    return await invoke('get_wallet_id', {})
}

export async function getWallets(): Promise<WalletData[]> {
    return await invoke('get_wallets', {})
}

export async function removeWallet(id: number) {
    await invoke('remove_wallet', { id })
}

export async function getWalletAddress(index: number): Promise<string> {
    return await invoke('wallet_address', { index: index })
}
export async function ExportWallet(password: string, id: number): Promise<string[]> {
    return await invoke('export_wallet', { password, id })
}

export async function resetToHeight(height:number): Promise<string[]> {
    return await invoke('reset_to_height', { height})
}



