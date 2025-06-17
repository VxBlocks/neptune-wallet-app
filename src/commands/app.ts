import { invoke } from "@tauri-apps/api/core";

export async function run_rpc_server() {
    await invoke('run_rpc_server', {})
}

export async function persist_store_execute(sql: string): Promise<[{}]> {
    return await invoke('persist_store_execute', { sql })
}

export async function snapshot_dir(): Promise<string> {
    return await invoke('snapshot_dir', {})
}


export async function stop_rpc_server() {
    await invoke('stop_rpc_server', {})
}

export async function get_server_url(): Promise<string> {
    return await invoke('get_server_url', {})
}

export interface BuildInfo {
    time: string,
    commit: string,
}
export interface UpdateInfo {
    version: string,
    url: string,
}


export async function get_build_info(): Promise<BuildInfo> {
    return await invoke('get_build_info', {})
}

export async function get_update_info(): Promise<UpdateInfo> {
    return await invoke('update_info', {})
}