use clap::Parser;
use std::fs::File;
use std::net::{SocketAddr, TcpListener};
use std::os::fd::AsRawFd;
use tokio::net::TcpStream;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("failed to switch namespace: {0}")]
    NsSet(nix::errno::Errno),

    #[error("failed to open namespace file: {0}")]
    NsOpen(std::io::Error),

    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "[::]:8080")]
    listen: SocketAddr,

    #[clap(short, long)]
    target: SocketAddr,

    #[clap(short, long)]
    namespace: String,
}

fn app(args: Args) -> Result<()> {
    let listener = TcpListener::bind(args.listen)?;
    let ns = File::open(&args.namespace).map_err(Error::NsOpen)?;
    nix::sched::setns(ns.as_raw_fd(), nix::sched::CloneFlags::CLONE_NEWNET)
        .map_err(Error::NsSet)?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ncc(args, listener))
}

async fn ncc(args: Args, listener: TcpListener) -> Result<()> {
    listener.set_nonblocking(true)?;
    let listener = tokio::net::TcpListener::from_std(listener)?;

    while let Ok((incoming, _)) = listener.accept().await {
        tokio::spawn(handle(incoming, args.target));
    }

    Ok(())
}

async fn handle(mut incoming: TcpStream, target: SocketAddr) -> Result<()> {
    let mut target = TcpStream::connect(target).await?;

    tokio::io::copy_bidirectional(&mut incoming, &mut target)
        .await
        .map(|(_, _)| ())
        .map_err(Error::IO)
}

fn main() {
    let args = Args::parse();

    if let Err(err) = app(args) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
