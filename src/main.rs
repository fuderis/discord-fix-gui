#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use app::{ prelude::*, Runner };
use tauri::WindowEvent;

static RUNNER: Lazy<Runner> = Lazy::new(|| Runner::new());

/// Run Discord Fix process
#[tauri::command]
async fn start_process() -> StdResult<String, String> {    
    let name = CONFIG.lock().await.active_bat.clone();
    
    // run process:
    match RUNNER.run().await {
        Ok(_) => {
            info!("The process '{name}' is started!");
            Ok(name)
        }

        Err(e) => {
            err!("Failed to run the '{name}.bat' process: {e}");
            Err(e.to_string())
        }
    }
}

/// Stop Discord Fix process
#[tauri::command]
async fn stop_process() -> StdResult<String, String> {    
    let name = CONFIG.lock().await.active_bat.clone();
    
    // stop process:
    match RUNNER.stop().await {
        Ok(_) => {
            info!("The process '{name}' is stopped!");
            Ok(name)
        }

        Err(e) => {
            err!("Failed to stop the '{name}.bat' process: {e}");
            Err(e.to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // init logger:
    LOGGER.init()?;

    // running fix:
    RUNNER.run().await?;
    
    // run ui:
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_process,
            stop_process
        ])
        .setup(|app| {
            let handle =  app.app_handle().clone();
            let window = handle.get_webview_window("main").unwrap();
            
            // init app handler:
            *APP_HANDLE.lock().unwrap() = Some(handle.clone());
            
            // init tray-icon:
            *SYSTEM_TRAY.lock().unwrap() = Some(Tray::new());
            
            // window events:
            window.on_window_event(move |event| {
                let window = handle.get_webview_window("main").unwrap();
                
                match event {
                    // if window closes:
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();

                        // closing process:
                        tokio::task::block_in_place(|| {
                            tauri::async_runtime::block_on(async {
                                if let Err(e) = RUNNER.stop().await {
                                    err!("Failed to close process: {e}");
                                }
                            });
                        });

                        // saving logs:
                        LOGGER.save().unwrap();

                        // removing tray:
                        if let Some(tray) = SYSTEM_TRAY.lock().unwrap().take() {
                            tray.remove();
                        }

                        // closing program:
                        handle.exit(0);
                    }

                    // if window minimized:
                    WindowEvent::Resized(_) => {
                        if window.is_minimized().unwrap_or(false) {
                            window.hide().unwrap();
                        }
                    }

                    _ => {}
                }
            });

            // hiding app on start:
            window.hide().unwrap();
            
            Ok(())
        })
        .plugin(tauri_plugin_prevent_default::Builder::new()
            .with_flags(tauri_plugin_prevent_default::Flags::empty())
            .build()
        )
        .run(tauri::generate_context!())?;

    Ok(())
}
