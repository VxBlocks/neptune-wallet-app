import { invoke } from "@tauri-apps/api/core";

export async function get_logs(): Promise<string[]> {
    return await invoke('get_logs', {})
}

export async function clear_logs() {
    await invoke('clear_logs', {})
}

export async function log(level: string, message: string) {
    await invoke('log', { level: level, message: message })
}

export async function set_log_level(level: string) {
    await invoke('set_log_level', { level: level })
}

export async function get_log_level(): Promise<string> {
    return await invoke('get_log_level', {})
}