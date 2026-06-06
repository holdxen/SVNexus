// svn+ssh://host/path	ssh -q -- host svnserve -t
// svn+ssh://user@host/path	ssh -q -- user@host svnserve -t
// svn+ssh://user@host:2222/path	ssh -q -- user@host:2222 svnserve -t
// svn+ssh://user:pass@host/path	ssh -q -- user:pass@host svnserve -t

use clap::Parser;

#[derive(Parser)]
struct Command {
    #[arg(short)]
    quiet: bool,

    #[arg(long)]
    rpc: String,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    let cmd = Command::parse();

    if cmd.args.len() < 1 {
        tracing::info!("Invalid command call");
        return;
    }
}
