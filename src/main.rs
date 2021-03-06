#![warn(clippy::all, clippy::pedantic)]

use {
    agis::CONFIG,
    std::{env, net::TcpListener, num::NonZeroUsize, process, thread},
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
    println!("Starting up thread pool");
    let threads = NonZeroUsize::new(CONFIG.threads).unwrap();
    let pool = agis::ThreadPool::new(threads);
    let listener = TcpListener::bind(format!("{}:{}", CONFIG.address.ip, CONFIG.address.port))?;
    println!(
        "Binding to address {} on port {}.",
        CONFIG.address.ip, CONFIG.address.port
    );
    let listener1 = match CONFIG.address1 {
        Some(ref a) => {
            let l = TcpListener::bind(format!("{}:{}", a.ip, a.port))?;
            println!("Binding to address {} on port {}.", a.ip, a.port);
            Some(l)
        },
        None => None,
    };
    unsafe {
        agis::init_logs((*user).pw_uid, (*group).gr_gid)?;
        agis::privdrop(user, group)?;
    }
    println!("Priviledges dropped, listening for incoming connections.");
    if let Some(ls) = listener1 {
        thread::spawn(move || {
            let pool = agis::ThreadPool::new(threads);
            for stream in ls.incoming() {
                let stream = stream.unwrap();
                pool.execute(|| {
                    if let Err(e) = agis::handle_connection(stream) {
                        eprintln!("{e}");
                    }
                });
            }
        });
    }
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
