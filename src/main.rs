#![warn(clippy::all, clippy::pedantic)]
use {
    agis::CONFIG,
    std::{env, net::TcpListener, num::NonZeroUsize, process},
};

fn main() -> std::io::Result<()> {
    let matches = agis::options().unwrap();
    if matches.opt_present("h") {
        agis::usage();
        process::exit(0);
    }
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        let prog = env!("CARGO_PKG_NAME");
        let prog = prog[0..1].to_uppercase() + &prog[1..];
        eprintln!("{} must be started as the root user.", prog);
        process::exit(1);
    }
    let user = CONFIG.getpwnam()?;
    let group = CONFIG.getgrnam()?;
    let listener = TcpListener::bind(format!("{}:{}", CONFIG.address, CONFIG.port))?;
    println!(
        "Binding to address {} on port {}.",
        CONFIG.address, CONFIG.port
    );
    unsafe {
        agis::init_logs((*user).pw_uid, (*group).gr_gid)?;
        agis::privdrop(user, group)?;
    }
    println!("Starting up thread pool");
    let threads = NonZeroUsize::new(CONFIG.threads).unwrap();
    let pool = agis::ThreadPool::new(threads);
    println!("Priviledges dropped, listening for incoming connections.");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            if let Err(e) = agis::handle_connection(stream) {
                eprintln!("{e}");
            }
        });
    }
    Ok(())
}
