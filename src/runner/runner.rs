use crate::prelude::*;
use std::{ fs, process::{ Command, Stdio, Child } };
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

// Discord Fix runner
#[derive(Debug)]
pub struct Runner {
    enabled: Arc<Mutex<bool>>,
    to_close: Arc<Mutex<bool>>,
    process: Arc<Mutex<Option<Child>>>
}

impl Runner {
    // Creates a new Discord Fix runner
    pub fn new() -> Self {
        Self {
            to_close: Arc::new(Mutex::new(false)),
            enabled: Arc::new(Mutex::new(false)),
            process: Arc::new(Mutex::new(None))
        }
    }

    // Read .bat file arguments
    fn parse_bat_args(bat_name: &str) -> Result<Vec<String>> {
        let path = path!("/bin/DiscordFix/pre-configs/{bat_name}.bat");
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

        dbg!(&vars);

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

        dbg!(&args);
        Ok(args)
    }

    // Runs discord fix
    pub async fn run(&self) -> Result<()> {
        *self.enabled.lock().await = true;

        // get active bat name:
        let bat_name = CONFIG.lock().await.active_bat.clone();
        let args = Self::parse_bat_args(&bat_name)?;

        // run Discord Fix process:
        *self.process.lock().await = Some(
            Command::new(&path!("/bin/DiscordFix/bin/winws.exe"))
                .args(&args)
                // .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
        );
        
        let to_close = self.to_close.clone();
        let enabled = self.enabled.clone();
        let process = self.process.clone();
        
        // waiting closing process:
        tokio::spawn(async move {
            loop {
                // checking status for enabled:
                if *to_close.lock().await {
                    break;
                }

                sleep(Duration::from_millis(100)).await;
            }

            // killing Discord Fix process:
            if let Some(mut process) = process.lock().await.take() {
                let _ = process.kill();
                let _ = process.wait();
                *enabled.lock().await = false;
            }
        });

        Ok(())
    }

    // Stops discord fix
    pub async fn stop(&self) -> Result<()> {
        *self.to_close.lock().await = true;

        while *self.enabled.lock().await {
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }
}
