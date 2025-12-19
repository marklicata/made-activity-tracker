use std::process::{Child, Command, Stdio};
use std::net::TcpListener;
use anyhow::{Result, anyhow};
use std::env;
use std::path::PathBuf;

pub struct AmplifierSidecar {
    process: Option<Child>,
    pub port: u16,
    pub auth_token: String,
}

impl AmplifierSidecar {
    pub fn new() -> Self {
        let auth_token = uuid::Uuid::new_v4().to_string();

        Self {
            process: None,
            port: 0,
            auth_token,
        }
    }

    pub fn start(&mut self, db_path: PathBuf) -> Result<()> {
        // Find available port
        self.port = self.find_available_port()?;

        // Get path to Python server
        let server_path = self.get_server_path()?;

        println!("Starting Amplifier sidecar on port {}", self.port);
        println!("Server path: {}", server_path);
        println!("Database path: {:?}", db_path);

        // Start Python server using venv Python
        let python_exe = self.get_python_path()?;
        println!("Python executable: {}", python_exe);

        let mut cmd = Command::new(&python_exe);
        cmd.arg(&server_path)
            .env("AMPLIFIER_PORT", self.port.to_string())
            .env("AMPLIFIER_AUTH_TOKEN", &self.auth_token)
            .env("DATABASE_PATH", db_path.to_string_lossy().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Pass through API keys
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            cmd.env("ANTHROPIC_API_KEY", key);
            println!("Using ANTHROPIC_API_KEY");
        }
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            cmd.env("OPENAI_API_KEY", key);
            println!("Using OPENAI_API_KEY");
        }

        let child = cmd.spawn()
            .map_err(|e| anyhow!("Failed to spawn Python process: {}. Python path: {}", e, python_exe))?;
        self.process = Some(child);

        // Wait for server to be ready
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
        }
        Ok(())
    }

    fn find_available_port(&self) -> Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        Ok(port)
    }

    fn get_python_path(&self) -> Result<String> {
        // In development, use venv Python
        if cfg!(debug_assertions) {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let python_path = PathBuf::from(manifest_dir)
                .join("amplifier-tools")
                .join(".venv")
                .join(if cfg!(windows) { "Scripts" } else { "bin" })
                .join(if cfg!(windows) { "python.exe" } else { "python" });

            if !python_path.exists() {
                return Err(anyhow!("Python venv not found at {:?}. Run 'uv venv' in amplifier-tools/", python_path));
            }

            Ok(python_path.to_string_lossy().to_string())
        } else {
            // In production, use bundled Python
            Ok("python".to_string())
        }
    }

    fn get_server_path(&self) -> Result<String> {
        // In development
        if cfg!(debug_assertions) {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let server_path = PathBuf::from(manifest_dir)
                .join("amplifier-tools")
                .join("src")
                .join("made_activity_tools")
                .join("server.py");

            if !server_path.exists() {
                return Err(anyhow!("Server script not found at {:?}", server_path));
            }

            Ok(server_path.to_string_lossy().to_string())
        } else {
            // In production, bundle with app
            let exe_dir = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow!("Failed to get exe directory"))?
                .to_path_buf();

            Ok(exe_dir
                .join("amplifier-tools")
                .join("server.py")
                .to_string_lossy()
                .to_string())
        }
    }
}

impl Drop for AmplifierSidecar {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
