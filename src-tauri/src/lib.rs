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
pub mod prover;
mod rpc;
pub mod rpc_client;
mod service;
#[cfg(feature = "gui")]
mod session_store;
pub mod wallet;
pub use neptune_privacy;

#[cfg(feature = "gui")]
pub fn add_commands<R: tauri::Runtime>(app: tauri::Builder<R>) -> tauri::Builder<R> {
    app.invoke_handler(tauri::generate_handler![
        command::commands::add_wallet,
        command::commands::delete_cache,
        command::commands::export_wallet,
        command::commands::generate_snapshot_file,
        command::commands::get_disk_cache,
        command::commands::get_network,
        command::commands::get_remote_rest,
        command::commands::get_wallet_id,
        command::commands::get_wallets,
        command::commands::has_password,
        command::commands::input_password,
        command::commands::list_cache,
        command::commands::remove_wallet,
        command::commands::reset_to_height,
        command::commands::set_disk_cache,
        command::commands::set_network,
        command::commands::set_password,
        command::commands::set_remote_rest,
        command::commands::set_wallet_id,
        command::commands::snapshot_dir,
        command::commands::try_password,
        command::commands::wallet_address,
        rpc::commands::avaliable_utxos,
        rpc::commands::current_wallet_address,
        rpc::commands::forget_tx,
        rpc::commands::get_server_url,
        rpc::commands::get_tip_height,
        rpc::commands::history,
        rpc::commands::pending_transactions,
        rpc::commands::run_rpc_server,
        rpc::commands::send_to_address,
        rpc::commands::stop_rpc_server,
        rpc::commands::sync_state,
        rpc::commands::wallet_balance,
        os::is_win11,
        os::os_info,
        os::platform,
        logger::clear_logs,
        logger::get_log_level,
        logger::get_logs,
        logger::log,
        logger::set_log_level,
        session_store::command::persist_store_execute,
        session_store::command::session_store_del,
        session_store::command::session_store_get,
        session_store::command::session_store_set,
        service::app::get_build_info,
        service::app::update_info,
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
