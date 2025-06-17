import { BlockCacheFile } from "@/store/types";
import { invoke } from "@tauri-apps/api/core"; 
export async function get_network(): Promise<string> {
    return await invoke('get_network', {})
}

export async function set_network(network: string) {
    return await invoke('set_network', { network: network })
}

export async function get_rest_url(): Promise<string> {
    return await invoke('get_remote_rest', {})
}

export async function set_rest_url(rest_url: string) {
    return await invoke('set_remote_rest', { rest: rest_url })
}

export async function set_disk_cache(enable: boolean) {
    await invoke('set_disk_cache', { enabled: enable })
}

export async function get_disk_cache(): Promise<boolean> {
    return await invoke('get_disk_cache', {})
}
export async function get_list_cache(): Promise<BlockCacheFile[]> {
    return await invoke('list_cache', {})
}
export async function delete_cache(path: String): Promise<boolean> {
    return await invoke('delete_cache', { path })
}

