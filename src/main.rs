#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use app::{ prelude::*, Runner };
use tauri::WindowEvent;
use std::fs;

static RUNNER: Lazy<Runner> = Lazy::new(|| Runner::new());

// Convert error to string
fn err_to_str(e: impl std::error::Error + Send + Sync + 'static) -> String {
    let e = Error::FailedReadBatsList(Box::new(e));
    err!("{e}");
    
    e.to_string()
}

/// Get process status
#[tauri::command]
async fn get_status() -> StdResult<bool, String> {
    Ok(RUNNER.is_enabled().await)
}

/// Get .bat files list
#[tauri::command]
async fn get_bats_list() -> StdResult<Vec<String>, String> {
    let mut bats = vec![];
    let active_bat = CONFIG.lock().unwrap().active_bat.clone();

    for entry in fs::read_dir(path!("/bin/DiscordFix/pre-configs")).map_err(err_to_str)? {
        let path = entry.map_err(err_to_str)?.path();
        
        if path.is_dir() { continue }

        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("bat") {
                if let Some(file_name) = path.file_name() {
                    let file_name = file_name.to_string_lossy()
                        .replace("\"", "&quot;")
                        .replace("<", "&lt;")
                        .replace(">", "&gt;");
                    let uniq_id = uniq_id();
                    let checked = if &file_name == &active_bat {"checked"}else{""};

                    let bat_html = fmt!(r#"
                        <div>
                            <input id="bat-name-{uniq_id}" name="bat-name" value="{file_name}" type="radio" {checked}>
                            <label for="bat-name-{uniq_id}">{file_name}</label>
                        </div>
                    "#);
                    
                    bats.push(bat_html);
                }
            }
        }
    }

    Ok(bats)
}

/// Set active .bat file
#[tauri::command]
async fn set_active_bat(bat_name: String) -> StdResult<(), String> {
    let mut config = CONFIG.lock().unwrap();
    config.active_bat = bat_name
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">");

    config.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Run discord fix process
#[tauri::command]
async fn run_process() -> StdResult<String, String> {    
    let name = CONFIG.lock().unwrap().active_bat.clone();
    
    // run process:
    match RUNNER.run().await {
        Ok(_) => {
            info!("The process '{name}' is runned!");
            Ok(name)
        }

        Err(e) => {
            err!("Failed to run the '{name}.bat' process: {e}");
            Err(e.to_string())
        }
    }
}

/// Stop discord fix process
#[tauri::command]
async fn stop_process() -> StdResult<String, String> {    
    let name = CONFIG.lock().unwrap().active_bat.clone();
    
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

static IS_RUNNED: StdMutex<bool> = StdMutex::new(false);

#[tokio::main]
async fn main() -> Result<()> {
    // init logger:
    LOGGER.init()?;

    // running process:
    RUNNER.run().await.unwrap();
    *IS_RUNNED.lock().unwrap() = RUNNER.is_enabled().await;
    
    // run ui:
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_bats_list,
            set_active_bat,
            run_process,
            stop_process
        ])
        .setup(|app| {
            let handle =  app.app_handle().clone();
            let window = handle.get_webview_window("main").unwrap();
            
            // init app handler:
            *APP_HANDLE.lock().unwrap() = Some(handle.clone());
            
            // init tray-icon:
            *SYSTEM_TRAY.lock().unwrap() = Some(Tray::new(if *IS_RUNNED.lock().unwrap() {"icon.ico"}else{"icon2.ico"}));
            
            // window events:
            window.on_window_event(move |event| {
                let window = handle.get_webview_window("main").unwrap();
                
                match event {
                    // if window closes:
                    WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();

                        // closing process:
                        warn!("Closing 'winws.exe' process before closing program..");
                        RUNNER.stop_unsafe();

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
