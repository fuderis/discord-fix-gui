use crate::prelude::*;
use std::{ fs, process::{ Command, Stdio, Child } };
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// Discord fix runner
#[derive(Debug)]
pub struct Runner {
    enabled: Arc<StdMutex<bool>>,
    to_close: Arc<StdMutex<bool>>,
    closed_unsafe: Arc<StdMutex<bool>>,
    process: Arc<StdMutex<Option<Child>>>
}

impl Runner {
    // Creates a new discord fix runner
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(StdMutex::new(false)),
            to_close: Arc::new(StdMutex::new(false)),
            closed_unsafe: Arc::new(StdMutex::new(false)),
            process: Arc::new(StdMutex::new(None))
        }
    }

    // Returns process status
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    // Read .bat file arguments
    fn parse_bat_args(bat_name: &str) -> Result<Vec<String>> {
        let path = path!("/bin/DiscordFix/pre-configs/{bat_name}{}", if !bat_name.ends_with(".bat") {".bat"}else{""});
        let content = fs::read_to_string(path)?;

        // parse variables:
        let mut vars = map!{};
        let var_re = re!(r#"set ([\w_]+)=%~dp0\.\.\\(.+)\n"#);
        let mut caps_iter = var_re.captures_iter(&content);

        while let Some(caps) = caps_iter.next() {
            let (name, value) = (
                fmt!("%{}%", &caps[1]),
                path!("\\bin\\DiscordFix\\{}", &caps[2]).to_str().unwrap().to_owned()
            );
            vars.insert(name, value);
        }

        // parse arguments:
        let args = if let Some((_first, other)) = content.split_once("winws.exe\" ") {
            other.trim().replace("\"", "").split_whitespace()
                .into_iter()
                .map(|s| {
                    let mut s = s.trim().to_string();

                    for (k, v) in &vars {
                        s = s.replace(k, v);
                    }

                    s
                })
                .collect::<Vec<_>>()
        } else {
            return Err(Error::FailedParseBatFile(bat_name.to_owned()).into())
        };

        Ok(args)
    }

    // Runs discord fix
    pub async fn run(&self) -> Result<()> {
        // get active bat name:
        let bat_name = CONFIG.lock().unwrap().active_bat.clone();
        let args = Self::parse_bat_args(&bat_name)?;

        // run discord fix process:
        let mut process = Command::new(&path!("/bin/DiscordFix/bin/winws.exe"))
                .args(&args)
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;

        // check process to alive:
        if let Some(status) = process.try_wait()? {
            if !status.success() {
                return Err(Error::FailedRunWinwsProcess.into());
            }
        }

        *self.process.lock().unwrap() = Some(process);

        emit_event("process-runned", HashMap::<&str, &str>::new());
        *self.enabled.lock().unwrap() = true;
        if let Some(tray) = SYSTEM_TRAY.lock().unwrap().as_mut() {
            tray.set_icon("icon.ico")?;
        }
        
        let enabled = self.enabled.clone();
        let to_close = self.to_close.clone();
        let closed_unsafe = self.closed_unsafe.clone();
        let process = self.process.clone();
        
        // waiting closing process:
        tokio::spawn(async move {
            loop {
                // checking status for enabled:
                if *to_close.lock().unwrap() {
                    break;
                }

                sleep(Duration::from_millis(100)).await;
            }

            // killing discord fix process:
            if let Some(mut process) = process.lock().unwrap().take() {
                let _ = process.kill();
                let _ = process.wait();

                if !*closed_unsafe.lock().unwrap() {
                    emit_event("process-stopped", HashMap::<&str, &str>::new());
                    
                    *to_close.lock().unwrap() = false;
                    *enabled.lock().unwrap() = false;

                    if let Some(tray) = SYSTEM_TRAY.lock().unwrap().as_mut() {
                        tray.set_icon("icon2.ico").unwrap();
                    }
                }
            }
        });

        Ok(())
    }

    // Stops discord fix
    pub async fn stop(&self) -> Result<()> {
        *self.to_close.lock().unwrap() = true;

        while *self.enabled.lock().unwrap() && self.process.lock().unwrap().is_some() {
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    // Stops discord fix (unsafe variant)
    pub fn stop_unsafe(&self) {
        *self.to_close.lock().unwrap() = true;
        *self.closed_unsafe.lock().unwrap() = true;
        std_sleep(Duration::from_millis(1000));
    }
}
