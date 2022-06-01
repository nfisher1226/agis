use std::{env, fs, io::Error, path::PathBuf, process};

fn copy_bin() -> Result<(), Error> {
    println!("Copying binary:");
    let bindir: PathBuf = ["target", "dist", "bin"].iter().collect();
    if !bindir.exists() {
        fs::create_dir_all(&bindir)?;
    }
    let mut outfile = bindir;
    outfile.push("agis");
    let infile: PathBuf = ["target", "release", "agis"].iter().collect();
    if !infile.exists() {
        eprintln!("Error: you must run \"cargo build --release\" first");
    }
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    Ok(())
}

fn copy_config() -> Result<(), Error> {
    println!("Copying config:");
    let confdir: PathBuf = ["target", "dist", "etc", "agis"].iter().collect();
    if !confdir.exists() {
        fs::create_dir_all(&confdir)?;
    }
    let mut outfile = confdir;
    outfile.push("config.ron");
    let infile: PathBuf = ["conf", "config.ron"].iter().collect();
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    Ok(())
}

fn copy_service() -> Result<(), Error> {
    println!("Copying service file:");
    let servicedir: PathBuf = ["target", "dist", "etc", "systemd", "system"].iter().collect();
    if !servicedir.exists() {
        fs::create_dir_all(&servicedir)?;
    }
    let mut outfile = servicedir;
    outfile.push("spartan.service");
    let infile: PathBuf = ["conf", "spartan.service"].iter().collect();
    fs::copy(&infile, &outfile)?;
    println!("    {} -> {}", infile.display(), outfile.display());
    Ok(())
}

fn usage() {
    println!("Usage: xtask dist");
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        usage();
        process::exit(0);
    }
    if &args[1] == "dist" {
        let outdir: PathBuf = ["target", "dist"].iter().collect();
        if outdir.exists() {
            fs::remove_dir_all(&outdir)?;
        }
        copy_bin()?;
        copy_config()?;
        copy_service()?;
    } else {
        usage();
    }
    Ok(())
}
