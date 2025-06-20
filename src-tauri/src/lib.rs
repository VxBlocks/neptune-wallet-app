// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(new_range_api)]
#![feature(linked_list_retain)]

#[cfg(not(feature = "gui"))]
mod cli;
mod command;
mod config;
#[cfg(feature = "gui")]
mod gui;
mod logger;
mod os;
mod rpc;
mod rpc_client;
mod service;
#[cfg(feature = "gui")]
mod session_store;
pub mod wallet;

#[cfg(feature = "gui")]
use command::commands::{
    add_wallet, delete_cache, export_wallet, generate_snapshot_file, get_disk_cache, get_network,
    get_remote_rest, get_wallet_id, get_wallets, has_password, input_password, list_cache,
    remove_wallet, reset_to_height, set_disk_cache, set_network, set_password, set_remote_rest,
    set_wallet_id, snapshot_dir, try_password, wallet_address,
};
#[cfg(feature = "gui")]
use logger::{clear_logs, get_log_level, get_logs, log, set_log_level};
#[cfg(feature = "gui")]
use os::{is_win11, os_info, platform};
#[cfg(feature = "gui")]
use rpc::commands::{
    avaliable_utxos, current_wallet_address, forget_tx, get_server_url, get_tip_height, history,
    pending_transactions, run_rpc_server, send_to_address, stop_rpc_server, sync_state,
    wallet_balance,
};
#[cfg(feature = "gui")]
use service::app::{get_build_info, update_info};

#[cfg(feature = "gui")]
use session_store::command::{
    persist_store_execute, session_store_del, session_store_get, session_store_set,
};

#[cfg(feature = "gui")]
pub fn add_commands<R: tauri::Runtime>(app: tauri::Builder<R>) -> tauri::Builder<R> {
    app.invoke_handler(tauri::generate_handler![
        // myhandler,
        get_logs,
        clear_logs,
        is_win11,
        os_info,
        platform,
        get_server_url,
        get_build_info,
        get_network,
        get_remote_rest,
        set_network,
        set_remote_rest,
        get_wallet_id,
        get_wallets,
        add_wallet,
        remove_wallet,
        export_wallet,
        wallet_address,
        set_wallet_id,
        stop_rpc_server,
        run_rpc_server,
        input_password,
        set_password,
        has_password,
        try_password,
        set_log_level,
        get_log_level,
        log,
        session_store_get,
        session_store_set,
        session_store_del,
        persist_store_execute,
        generate_snapshot_file,
        snapshot_dir,
        update_info,
        set_disk_cache,
        get_disk_cache,
        reset_to_height,
        list_cache,
        delete_cache,
        sync_state,
        wallet_balance,
        current_wallet_address,
        history,
        avaliable_utxos,
        send_to_address,
        pending_transactions,
        forget_tx,
        get_tip_height,
    ])
}

pub fn run() {
    #[cfg(feature = "gui")]
    gui::run();
    #[cfg(not(feature = "gui"))]
    {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            crate::logger::setup_logger(None).unwrap();
            cli::run().await;
        })
    }
}
