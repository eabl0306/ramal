//! El binario `ramal-api` lanzado como proceso, como lo lanza un contenedor.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

/// Ruta del binario que cargo compila para los tests de este paquete.
const BINARY: &str = env!("CARGO_BIN_EXE_ramal-api");

/// Base de datos a la que nadie responde: el puerto 1 rechaza la conexion.
pub const UNREACHABLE_DATABASE: &str = "postgres://ramal:ramal@127.0.0.1:1/ramal";

/// Orden del binario sin las variables `RAMAL__` del entorno de los tests.
///
/// `just` carga `.env` y la configuracion rechaza claves desconocidas, asi que
/// heredar el entorno haria que el resultado dependiera de la maquina. El resto
/// del entorno se hereda a proposito: `cargo llvm-cov` necesita el suyo para
/// medir tambien la cobertura del binario.
pub fn command() -> Command {
    let mut command = Command::new(BINARY);
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("RAMAL__") {
            command.env_remove(key);
        }
    }
    command
}

/// Ejecuta el binario con los argumentos dados y espera a que termine.
pub fn run(arguments: &[&str]) -> Output {
    command()
        .args(arguments)
        .output()
        .expect("the binary should start")
}

/// Puerto libre en este momento. Puede ocuparlo otro proceso antes de que el
/// binario lo abra, pero en una maquina de tests es improbable.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .expect("the system should hand out a free port")
}

/// Servidor en marcha. Al soltarlo se mata si sigue vivo, para no dejar
/// procesos colgados si un test falla a medias.
pub struct ApiProcess {
    child: Child,
    port: u16,
}

impl ApiProcess {
    /// Arranca el servidor en un puerto libre contra la base dada.
    pub fn start(database_url: &str) -> Self {
        let port = free_port();
        let child = command()
            .env("RAMAL__SERVER__HOST", "127.0.0.1")
            .env("RAMAL__SERVER__PORT", port.to_string())
            .env("RAMAL__DATABASE__URL", database_url)
            .env("RAMAL__DATABASE__ACQUIRE_TIMEOUT_MS", "500")
            .env("RAMAL__TELEMETRY__LOG_LEVEL", "warn")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the binary should start");
        let process = Self { child, port };
        process.wait_until_listening();
        process
    }

    /// Puerto en el que escucha.
    pub const fn port(&self) -> u16 {
        self.port
    }

    fn wait_until_listening(&self) {
        let address = SocketAddr::from(([127, 0, 0, 1], self.port));
        let deadline = Instant::now() + Duration::from_secs(30);
        while TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_err() {
            assert!(
                Instant::now() < deadline,
                "ramal-api did not listen on {address} within 30s"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Manda SIGTERM, como un orquestador, y espera a que el proceso salga.
    pub fn terminate(mut self) -> ExitStatus {
        let status = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .expect("kill should run");
        assert!(status.success(), "kill -TERM failed");
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(exit) = self.child.try_wait().expect("the child can be polled") {
                return exit;
            }
            assert!(
                Instant::now() < deadline,
                "ramal-api did not stop within 20s of SIGTERM"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

impl Drop for ApiProcess {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
