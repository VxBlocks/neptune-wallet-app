#[cfg(desktop)]
use crate::rpc::commands::{get_server_url, stop_rpc_server};
use crate::session_store::Memstore;
use std::sync::Arc;

use tracing::*;

use tauri::Manager;

#[cfg(desktop)]
use tauri::{image::Image, App, Emitter};

#[cfg(desktop)]
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

#[cfg(desktop)]
const MENU_ITEM_QUIT: &str = "Quit";
#[cfg(desktop)]
const MENUITEM_COPY_ADDR: &str = "Copy server address";
#[cfg(desktop)]
const MENUITEM_SHOW: &str = "Show window";

#[cfg(desktop)]
#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(all(desktop, target_os = "linux"))]
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    info!("Starting Neptune Cash");

    let builder = tauri::Builder::default();
    let builder = super::add_commands(builder);
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init());

    let app = builder
        .setup(|app| {
            #[cfg(desktop)]
            build_tray_menu(app).unwrap();

            // #[cfg(desktop)]
            create_main_window(app.app_handle());
            let data_dir = app
                .path()
                .app_config_dir()
                .expect("failed to get app data dir");

            let config = tauri::async_runtime::block_on(async {
                let config = crate::config::Config::new(&data_dir).await.unwrap();

                crate::rpc_client::node_rpc_client()
                    .set_rest_server(config.get_remote_rest().await.unwrap());

                let level = config.get_log_level().await.unwrap();
                crate::logger::setup_logger(level).unwrap();
                config
            });

            let persist_store = tauri::async_runtime::block_on(
                crate::session_store::persist::PersisStore::new(&data_dir),
            )
            .unwrap();
            crate::service::manage(persist_store);

            crate::service::manage(Arc::new(config));
            let memstore = Memstore::new();
            crate::service::manage(memstore);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    //store app handle
    {
        let apphandle = app.app_handle().clone();

        crate::service::manage(apphandle);
    }

    #[cfg(desktop)]
    tauri::async_runtime::spawn(async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                async_before_exit().await;
                std::process::exit(0)
            }
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
                // we also shut down in case of error
                std::process::exit(1)
            }
        }
    });

    #[allow(unused_variables)]
    app.run(|app, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            api.prevent_exit();
        }
        tauri::RunEvent::Exit => before_exit(),
        _ => {}
    })
}

fn before_exit() {
    tauri::async_runtime::block_on(async_before_exit());
}

async fn async_before_exit() {
    println!("Received shutdown signal");
    let app = crate::service::get_state::<tauri::AppHandle>();
    if let Err(e) = stop_rpc_server().await {
        crate::service::app::error_dialog(
            &app,
            &format!("Failed to stop rpc server {}", &e.to_string()),
        );
    }

    println!("gracefully shutdown");
}

#[cfg(desktop)]
fn toggle_window_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or_default() {
            let _ = window.hide();
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            #[cfg(target_os = "macos")]
            let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        }
    } else {
        create_main_window(app);
    }
}

#[cfg(desktop)]
fn create_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let window =
        tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
            .title("")
            .inner_size(1100.0, 750.0)
            .min_inner_size(860.0, 630.0)
            .resizable(true)
            .visible(false)
            .decorations(true)
            .shadow(true)
            .build();

    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Ok(window) = window {
        #[cfg(target_os = "macos")]
        let _ = window.set_title_bar_style(tauri::TitleBarStyle::Overlay);
        #[cfg(not(target_os = "macos"))]
        let _ = window.set_decorations(false);
        setup_window(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(mobile)]
fn create_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
        .build()
        .unwrap();
}

#[cfg(desktop)]
fn setup_window<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>) {
    #[cfg(debug_assertions)] // only include this code on debug builds
    {
        _window.open_devtools();
    }

    #[cfg(any(windows, target_os = "macos"))]
    let _ = _window.set_shadow(true);
}

#[cfg(desktop)]
fn build_tray_menu(app: &mut App) -> anyhow::Result<()> {
    let show = MenuItemBuilder::with_id(MENUITEM_SHOW, MENUITEM_SHOW).build(app)?;
    let copy_addr = MenuItemBuilder::with_id(MENUITEM_COPY_ADDR, MENUITEM_COPY_ADDR).build(app)?;
    let quit = MenuItemBuilder::with_id(MENU_ITEM_QUIT, MENU_ITEM_QUIT)
        .accelerator("Cmd+Q")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show, &copy_addr, &quit])
        .build()?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .icon(Image::from_bytes(include_bytes!("../icons/tray.png")).unwrap())
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            MENUITEM_SHOW => {
                toggle_window_visibility(app);
            }
            MENU_ITEM_QUIT => {
                before_exit();
                std::process::exit(0);
            }
            MENUITEM_COPY_ADDR => tauri::async_runtime::block_on(async {
                if let Ok(addr) = get_server_url().await {
                    use tauri_plugin_clipboard_manager::ClipboardExt;
                    match app.clipboard().write_text(addr.clone()) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("failed to write clipboard: {}", e);
                        }
                    }
                }
            }),
            _ => (),
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = crate::service::get_state::<tauri::AppHandle>();
                toggle_window_visibility(&app);
            }
        })
        .build(app)?;

    Ok(())
}
