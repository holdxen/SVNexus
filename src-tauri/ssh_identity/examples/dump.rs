//! Dump the identity files this crate would try, in OpenSSH's order.
//!
//! Useful for eyeballing fidelity against `ssh -G [-F file] host`:
//!
//! ```sh
//! cargo run --manifest-path ssh_identity/Cargo.toml --example dump -- \
//!     --config ~/.ssh/config --host example.com --port 2222
//! ```

use ssh_identity::{discover, DiscoverOptions};

fn main() {
    let mut config: Option<String> = None;
    let mut host = String::from("example.com");
    let mut port: u16 = 22;
    let mut remote_user: Option<String> = None;
    let mut home: Option<String> = None;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut index = 0;
    while index < argv.len() {
        let flag = argv[index].clone();
        index += 1;
        let value = argv.get(index).cloned().unwrap_or_default();
        match flag.as_str() {
            "--config" => {
                config = Some(value);
                index += 1;
            }
            "--host" => {
                host = value;
                index += 1;
            }
            "--port" => {
                port = value.parse().unwrap_or(22);
                index += 1;
            }
            "--user" => {
                remote_user = Some(value);
                index += 1;
            }
            "--home" => {
                home = Some(value);
                index += 1;
            }
            other => eprintln!("unknown argument: {other}"),
        }
    }

    let home = match home {
        Some(h) => std::path::PathBuf::from(h),
        None => std::env::home_dir().expect("no home directory"),
    };
    let config_path = config.as_deref().map(std::path::Path::new);

    let found = discover(
        &DiscoverOptions::new(&home, &host, port)
            .remote_user(remote_user.as_deref())
            .config(config_path),
    );

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
