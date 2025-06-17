import { invoke } from "@tauri-apps/api/core";

export default class RustySessionStore {
    constructor() {
    }

    static async get(key: string): Promise<string | null> {
        return await invoke('session_store_get', { key: key })
    }

    static async set(key: string, value: string): Promise<void> {
        await invoke('session_store_set', { key: key, value: value })
    }

    static async remove(key: string): Promise<string | null> {
        return await invoke('session_store_del', { key: key })
    }
}