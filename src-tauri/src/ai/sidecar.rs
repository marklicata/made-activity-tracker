use std::process::{Child, Command, Stdio};
use std::net::TcpListener;
use std::io::{BufRead, BufReader};
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
        tracing::info!("=== Starting Amplifier Sidecar ===");
        tracing::info!("Database path: {:?}", db_path);

        // Find available port
        tracing::info!("Finding available port...");
        self.port = self.find_available_port()?;
        tracing::info!("✓ Found available port: {}", self.port);

        // Get path to Python server
        tracing::info!("Locating Python server script...");
        let server_path = self.get_server_path()?;
        tracing::info!("✓ Server path: {}", server_path);

        // Start Python server using venv Python
        tracing::info!("Locating Python executable...");
        let python_exe = self.get_python_path()?;
        tracing::info!("✓ Python executable: {}", python_exe);

        tracing::info!("Setting up environment variables...");

        // Convert database path if needed (for Windows Python accessing WSL filesystem)
        let db_path_str = db_path.to_string_lossy().to_string();
        let db_path_for_python = if db_path_str.starts_with("/home/") {
            // For Windows Python, convert WSL home paths to \\wsl$\Ubuntu\home\...
            // This allows Windows programs to access WSL filesystem
            let wsl_path = format!("\\\\wsl$\\Ubuntu{}", db_path_str);
            tracing::debug!("  Converting WSL home path to: {}", wsl_path);
            wsl_path
        } else {
            self.wsl_to_windows_path(&db_path_str)
        };

        let mut cmd = Command::new(&python_exe);
        cmd.arg(&server_path)
            .env("AMPLIFIER_PORT", self.port.to_string())
            .env("AMPLIFIER_AUTH_TOKEN", &self.auth_token)
            .env("DATABASE_PATH", &db_path_for_python)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        tracing::info!("  AMPLIFIER_PORT={}", self.port);
        tracing::info!("  AMPLIFIER_AUTH_TOKEN={}...", &self.auth_token[..8]);
        tracing::info!("  DATABASE_PATH={}", db_path_for_python);

        // Pass through API keys
        tracing::info!("About to check API keys...");
        let mut has_api_key = false;
        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            tracing::info!("Found ANTHROPIC_API_KEY");
            cmd.env("ANTHROPIC_API_KEY", key);
            tracing::info!("  ✓ ANTHROPIC_API_KEY is set");
            has_api_key = true;
        }
        if let Ok(key) = env::var("OPENAI_API_KEY") {
            tracing::info!("Found OPENAI_API_KEY");
            cmd.env("OPENAI_API_KEY", key);
            tracing::info!("  ✓ OPENAI_API_KEY is set");
            has_api_key = true;
        }

        if !has_api_key {
            tracing::warn!("  ⚠ No API keys found (ANTHROPIC_API_KEY or OPENAI_API_KEY)");
        }

        tracing::info!("About to spawn Python process...");
        eprintln!("DEBUG: About to spawn Python process at {}", python_exe);
        
        tracing::info!("Spawning Python server process...");
        let mut child = cmd.spawn()
            .map_err(|e| {
                eprintln!("DEBUG: Failed to spawn: {}", e);
                tracing::error!("✗ Failed to spawn Python process: {}", e);
                anyhow!("Failed to spawn Python process: {}. Python path: {}", e, python_exe)
            })?;

        eprintln!("DEBUG: Spawn succeeded!");
        let pid = child.id();
        tracing::info!("✓ Python server spawned with PID: {}", pid);
        eprintln!("DEBUG: PID is {}", pid);

        // Capture stderr/stdout and read in background threads to prevent pipe buffer deadlock
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[Python stderr] {}", line);
                    }
                }
            });
        }
        
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[Python stdout] {}", line);
                    }
                }
            });
        }

        // Wait for server to be ready - Flask needs time to start
        tracing::info!("Waiting 5 seconds for server to initialize...");
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Check if process is still running
        match child.try_wait() {
            Ok(Some(status)) => {
                tracing::error!("✗ Python process exited prematurely with status: {}", status);
                return Err(anyhow!("Python server failed to start - process exited with {}", status));
            }
            Ok(None) => {
                tracing::info!("✓ Python process is still running");
            }
            Err(e) => {
                tracing::warn!("⚠ Could not check process status: {}", e);
            }
        }

        self.process = Some(child);
        tracing::info!("✓ Amplifier sidecar startup complete");

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            tracing::info!("Stopping Amplifier sidecar (PID: {})", process.id());
            process.kill()?;
            tracing::info!("✓ Amplifier sidecar stopped");
        }
        Ok(())
    }

    fn find_available_port(&self) -> Result<u16> {
        tracing::debug!("  Binding to 127.0.0.1:0 to find available port...");
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| {
                tracing::error!("  ✗ Failed to bind to find available port: {}", e);
                anyhow!("Failed to find available port: {}", e)
            })?;
        let port = listener.local_addr()?.port();
        tracing::debug!("  Port {} is available", port);
        Ok(port)
    }

    fn get_python_path(&self) -> Result<String> {
        // In development, use venv Python
        if cfg!(debug_assertions) {
            tracing::debug!("  Running in development mode, looking for venv Python...");
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let venv_dir = PathBuf::from(manifest_dir)
                .join("amplifier-tools")
                .join(".venv");

            tracing::debug!("  Venv directory: {:?}", venv_dir);

            // Try Unix/WSL path first (bin/python) - WSL can't execute Windows .exe files
            let unix_python = venv_dir.join("bin").join("python");
            tracing::debug!("  Checking Unix/WSL path: {:?}", unix_python);
            if unix_python.exists() {
                tracing::debug!("  ✓ Found Unix/WSL Python executable");
                return Ok(unix_python.to_string_lossy().to_string());
            } else {
                tracing::debug!("  ✗ Unix/WSL Python not found");
            }

            // Try Windows path (Scripts/python.exe) - for native Windows execution
            let windows_python = venv_dir.join("Scripts").join("python.exe");
            tracing::debug!("  Checking Windows path: {:?}", windows_python);
            if windows_python.exists() {
                tracing::debug!("  ✓ Found Windows Python executable");
                return Ok(windows_python.to_string_lossy().to_string());
            } else {
                tracing::debug!("  ✗ Windows Python not found");
            }

            // Neither found
            tracing::error!("  ✗ Python venv not found in either location");
            Err(anyhow!(
                "Python venv not found. Tried:\n  - {:?}\n  - {:?}\nRun 'uv venv' in amplifier-tools/",
                unix_python, windows_python
            ))
        } else {
            tracing::debug!("  Running in production mode, using system Python");
            Ok("python".to_string())
        }
    }

    fn get_server_path(&self) -> Result<String> {
        // In development
        if cfg!(debug_assertions) {
            tracing::debug!("  Running in development mode, looking for server.py...");
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let server_path = PathBuf::from(manifest_dir)
                .join("amplifier-tools")
                .join("src")
                .join("made_activity_tools")
                .join("server.py");

            tracing::debug!("  Checking server path: {:?}", server_path);
            if !server_path.exists() {
                tracing::error!("  ✗ Server script not found at {:?}", server_path);
                return Err(anyhow!("Server script not found at {:?}", server_path));
            }

            tracing::debug!("  ✓ Found server.py");
            // Return path as-is - Python executable understands its native path format
            Ok(server_path.to_string_lossy().to_string())
        } else {
            tracing::debug!("  Running in production mode, looking for bundled server.py...");
            let exe_dir = std::env::current_exe()?
                .parent()
                .ok_or_else(|| anyhow!("Failed to get exe directory"))?
                .to_path_buf();

            let server_path = exe_dir
                .join("amplifier-tools")
                .join("server.py");
            tracing::debug!("  Production server path: {:?}", server_path);

            Ok(server_path.to_string_lossy().to_string())
        }
    }

    fn wsl_to_windows_path(&self, path: &str) -> String {
        // Convert /mnt/c/path to C:\path
        if path.starts_with("/mnt/") && path.len() > 6 {
            let drive_letter = &path[5..6].to_uppercase();
            let rest = &path[6..].replace('/', "\\");
            format!("{}:{}", drive_letter, rest)
        } else {
            path.to_string()
        }
    }
}

impl Drop for AmplifierSidecar {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
