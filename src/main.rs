pub mod config;
pub mod request;
pub mod response;
mod threadpool;

use {
    threadpool::ThreadPool,
    lazy_static::lazy_static,
    std::{
        env,
        error::Error,
        io::{BufReader, Write},
        process,
        net::{TcpListener, TcpStream},
        num::NonZeroUsize,
    },
};

pub use {
    config::Config,
    request::Request,
};

lazy_static! {
    static ref CONFIG: Config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Unable to load config: {e}");
            process::exit(1);
        }
    };
}

fn privdrop(user: *mut libc::passwd, group: *mut libc::group) -> std::io::Result<()> {
    if unsafe { libc::setgid((*group).gr_gid) } != 0 {
        eprintln!("privdrop: Unable to setgid of group: {}", &CONFIG.group);
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::setuid((*user).pw_uid) } != 0 {
        eprintln!("privdrop: Unable to setuid of user: {}", &CONFIG.user);
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(&stream);
    let _request = Request::try_from(&mut reader)?;
    stream.write("Hello world!".as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        eprintln!("Toe must be started as the root user.");
        process::exit(1);
    }
    let user = CONFIG.getpwnam()?;
    let group = CONFIG.getgrnam()?;
    //if CONFIG.chroot {
    //    unix::fs::chroot(&CONFIG.root)?;
    //}
    env::set_current_dir("/")?;
    let listener = TcpListener::bind(format!("{}:{}", CONFIG.address, CONFIG.port))?;
    println!(
        "Binding to address {} on port {}.",
        CONFIG.address, CONFIG.port
    );
    privdrop(user, group)?;
    println!("Starting up thread pool");
    let threads = NonZeroUsize::new(CONFIG.threads).unwrap();
    let pool = ThreadPool::new(threads);
    println!("Priviledges dropped, listening for incoming connections.");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("{e}");
            }
        });
    }
    Ok(())
}
