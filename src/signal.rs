use tokio::signal::ctrl_c;

pub async fn shutdown_signal() {
    ctrl_c().await.expect("install CTRL+C signal handler");
}
