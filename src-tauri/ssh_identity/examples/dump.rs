//! Dump the identity files this crate would try, in OpenSSH's order.
//!
//! Useful for eyeballing fidelity against `ssh -G [-F file] host`:
//!
//! ```sh
//! cargo run --manifest-path ssh_identity/Cargo.toml --example dump -- \
//!     --config ~/.ssh/config --host example.com --port 2222
//! ```
//!
//! Note that `ssh -G` echoes `IdentityFile` arguments *unexpanded*; use
//! `ssh -vv <host>` and the `identity file ...` debug lines to compare what
//! the tokens actually resolve to.

use ssh_identity::{discover, DiscoverOptions};

fn main() {
    let mut config: Option<String> = None;
    let mut system_config: Option<String> = None;
    let no_system_config = std::env::args().any(|a| a == "--no-system-config");
    let mut host = String::from("example.com");
    let mut port: u16 = 22;
    let mut remote_user: Option<String> = None;
    let mut local_user: Option<String> = None;
    let mut local_host: Option<String> = None;
    let mut uid: Option<u32> = None;
    let mut home: Option<String> = None;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut index = 0;
    while index < argv.len() {
        let flag = argv[index].clone();
        index += 1;
        if flag == "--no-system-config" {
            continue;
        }
        let value = argv.get(index).cloned().unwrap_or_default();
        index += 1;
        match flag.as_str() {
            "--config" => config = Some(value),
            "--system-config" => system_config = Some(value),
            "--host" => host = value,
            "--port" => port = value.parse().unwrap_or(22),
            "--user" => remote_user = Some(value),
            "--local-user" => local_user = Some(value),
            "--local-host" => local_host = Some(value),
            "--uid" => uid = value.parse().ok(),
            "--home" => home = Some(value),
            other => eprintln!("unknown argument: {other}"),
        }
    }

    let home = match home {
        Some(h) => std::path::PathBuf::from(h),
        None => std::env::home_dir().expect("no home directory"),
    };
    let config_path = config.as_deref().map(std::path::Path::new);
    let system_path = system_config.as_deref().map(std::path::Path::new);

    let mut options = DiscoverOptions::new(&home, &host, port)
        .remote_user(remote_user.as_deref())
        .local_user(local_user.as_deref())
        .local_host(local_host.as_deref())
        .uid(uid)
        .config(config_path);
    if no_system_config {
        options = options.system_config(Some(None));
    } else if system_path.is_some() {
        options = options.system_config(Some(system_path));
    }

    let found = discover(&options);

    for identity in &found.identities {
        println!(
            "identityfile={} user_provided={}",
            identity.path.display(),
            identity.user_provided
        );
    }
    for certificate in &found.certificates {
        println!("certificatefile={}", certificate.display());
    }
    match found.identities_only {
        Some(value) => println!("identitiesonly={value}"),
        None => println!("identitiesonly=(unset)"),
    }
    for warning in &found.warnings {
        eprintln!("warning: {warning}");
    }
}
