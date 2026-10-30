use libp2p::identity::Keypair;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct Process(Child);
impl Process {
    pub fn start(config: &Path) -> Self {
        let log = std::fs::File::create(config.with_extension("log")).unwrap();
        Self(
            Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
                .arg(config)
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .unwrap(),
        )
    }
    pub fn kill(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
    pub async fn terminate(&mut self, limit: Duration) {
        assert!(Command::new("kill")
            .args(["-TERM", &self.0.id().to_string()])
            .status()
            .unwrap()
            .success());
        let status = self.exited(limit).await;
        assert!(status.success(), "node exited {status}");
    }
    pub async fn exited(&mut self, limit: Duration) -> std::process::ExitStatus {
        let end = Instant::now() + limit;
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(Instant::now() < end, "node did not exit in time");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.kill();
    }
}
pub fn address(seed: u8) -> String {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let peer = Keypair::ed25519_from_bytes([seed; 32])
        .unwrap()
        .public()
        .to_peer_id();
    format!("/ip4/127.0.0.1/tcp/{port}/p2p/{peer}")
}
pub async fn until(path: &Path, expected: impl Fn(&str) -> bool) -> String {
    let end = Instant::now() + Duration::from_secs(25);
    loop {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        if expected(&text) {
            return text;
        }
        assert!(Instant::now() < end, "{}: {text}", path.display());
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}
