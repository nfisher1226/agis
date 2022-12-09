#![warn(clippy::all, clippy::pedantic)]

use {
    agis::{
        log::{Log, LogError},
        CONFIG,
    },
    std::{
        env,
        net::TcpListener,
        num::NonZeroUsize,
        process,
        sync::{mpsc::channel, Arc, Mutex},
        thread,
    },
};

fn main() -> std::io::Result<()> {
    // Get any CLI flags
    let matches = match agis::options() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}\n");
            agis::usage();
            process::exit(1);
        }
    };
    if matches.opt_present("h") {
        agis::usage();
        process::exit(0);
    }
    if matches.opt_present("v") {
        agis::version();
        process::exit(0);
    }
    // Make sure we're starting as root
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        let prog = env!("CARGO_PKG_NAME");
        let prog = prog[0..1].to_uppercase() + &prog[1..];
        eprintln!("{prog} must be started as the root user.");
        process::exit(1);
    }
    let user = CONFIG.getpwnam()?;
    let group = CONFIG.getgrnam()?;

    let _msg = "Starting up thread pool".to_string().log();
    let threads = NonZeroUsize::new(CONFIG.threads).unwrap();
    let pool = Arc::new(Mutex::new(agis::ThreadPool::new(threads)));
    let listener = TcpListener::bind(format!("{}:{}", CONFIG.address.ip, CONFIG.address.port))?;
    let _msg = format!(
        "Binding to address {} on port {}",
        CONFIG.address.ip, CONFIG.address.port
    )
    .log();
    // We can optionally start up a second listener, useful if we want to listen
    // on a second interface *or* listen to ipv4 and ipv6 simultaneously
    let listener1 = match CONFIG.address1 {
        Some(ref a) => {
            let l = TcpListener::bind(format!("{}:{}", a.ip, a.port))?;
            let _msg = format!("Binding to address {} on port {}", a.ip, a.port).log();
            Some(l)
        }
        None => None,
    };
    // Both of these functions call into libc, group them together so we only
    // have one unsafe block
    unsafe {
        agis::init_logs((*user).pw_uid, (*group).gr_gid)?;
        agis::privdrop(user, group)?;
    }
    let _msg = "Privileges dropped, listening for incoming connections"
        .to_string()
        .log();
    if let Some(ls) = listener1 {
        let pool = Arc::clone(&pool);
        thread::spawn(move || {
            for stream in ls.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        if let Err(e) = e.log_err() {
                            eprintln!("{e}");
                        }
                        continue;
                    }
                };
                if let Ok(pool) = pool.try_lock() {
                    pool.execute(|| {
                        if let Err(e) = agis::handle_connection(stream) {
                            if let Err(e) = e.log_err() {
                                eprintln!("{e}");
                            }
                        }
                    });
                }
            }
        });
    }
    {
        let pool = Arc::clone(&pool);
        thread::spawn(move || {
            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        if let Err(e) = e.log_err() {
                            eprintln!("{e}");
                        }
                        continue;
                    }
                };
                if let Ok(pool) = pool.try_lock() {
                    pool.execute(|| {
                        if let Err(e) = agis::handle_connection(stream) {
                            if let Err(e) = e.log_err() {
                                eprintln!("{e}");
                            }
                        }
                    });
                }
            }
        });
    }
    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Cannot send termination signal");
    })
    .expect("Cannot set signal handler");
    rx.recv()
        .expect("Could not receive message through channel");
    if let Ok(mut pool) = pool.try_lock() {
        pool.shutdown();
    }
    Ok(())
}
