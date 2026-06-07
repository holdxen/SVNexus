// svn+ssh://host/path	ssh -q -- host svnserve -t
// svn+ssh://user@host/path	ssh -q -- user@host svnserve -t
// svn+ssh://user@host:2222/path	ssh -q -- user@host:2222 svnserve -t
// svn+ssh://user:pass@host/path	ssh -q -- user:pass@host svnserve -t

use clap::Parser;
use svnexus_shared::*;
use tarpc::{client, context};

#[derive(Parser)]
struct Command {
    #[arg(short)]
    quiet: bool,

    #[arg(long)]
    rpc: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

struct ConnectOption {
    username: String,
    host: String,
    post: u16,
}

struct TunnelClient {
    client: TunnelServiceClient,
}

impl Command {
    async fn run(&self) -> anyhow::Result<()> {
        use std::path::Path;
        use tarpc::{serde_transport, tokio_serde::formats::Json};

        let transport = serde_transport::unix::connect(Path::new(&self.rpc), Json::default).await?;

        let client = TunnelServiceClient::new(client::Config::default(), transport).spawn();
        Ok(())
    }

    pub fn parse_connect_option(&self) -> anyhow::Result<ConnectOption> {
        if self.args.len() < 1 {
            anyhow::bail!("Failed to parse connect target");
        }

        let target = self.args[0].as_str();

        let username;
        let host;

        if target.contains('@') {
            let parts = target.split('@').collect::<Vec<_>>();
            if parts.len() != 2 {
                anyhow::bail!("Failed to parse connect target");
            }
            username = parts[0].to_string();
            host = parts[1].to_string();
        } else {
            username = whoami::username()?;
            host = target.to_string();
        }

        Ok(ConnectOption {
            username,
            host,
            post: 22,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();

    if cmd.args.len() < 1 {
        tracing::info!("Invalid command call");
        anyhow::bail!("Invalid command args");
    }

    Ok(())
}
