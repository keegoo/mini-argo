#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _client = connect_to_kubernetes().await?;

    println!("controller is running");
    tokio::signal::ctrl_c().await?;
    println!("shutting down");

    Ok(())
}

async fn connect_to_kubernetes() -> Result<kube::Client, Box<dyn std::error::Error>> {
    println!("loading Kubernetes configuration");
    let config = kube::Config::infer().await?;

    println!("connecting to Kubernetes API server");
    let client = kube::Client::try_from(config)?;

    let version = client.apiserver_version().await?;

    println!(
        "connected to Kubernetes API server (git_version = {})",
        version.git_version
    );

    Ok(client)
}
