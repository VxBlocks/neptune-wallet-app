import { invoke } from "@tauri-apps/api/core";

export async function isWin11() {
    return await invoke('is_win11');
}

export async function console_log(input: string) {
    await invoke('console_log', { abc: input })
}

export async function os_info(): Promise<Info> {
    return await invoke('os_info',)
}

export async function get_platform(): Promise<string> {
    return await invoke('platform',)
}

export interface Info {
    os_type: string
    version: any
    edition?: string
    codename?: string
    bitness: string
    architecture?: string
}
