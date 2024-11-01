use std::{
  net::IpAddr,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::{Parser, ValueEnum};
use serde::Serialize;

const E6: f64 = 1_000_000.0;
const PAYLOAD: &[u8] = &[];

#[derive(Parser)]
struct Cli {
  #[arg(help = "Server IP address")]
  server: IpAddr,

  #[arg(
    short,
    long,
    default_value_t = 100,
    help = "Interval between each ping (in ms)"
  )]
  interval: u64,

  #[arg(short, long, default_value = "human", help = "Output format")]
  format: Format,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
enum Format {
  Human,
  Csv,

  #[cfg(feature = "json")]
  Json,
}

#[derive(Serialize)]
struct Ping {
  timestamp: f64,
  rtt: f64,
}

impl Ping {
  fn new(rtt: Duration) -> Self {
    Self {
      timestamp: (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as f64)
        / E6,
      rtt: (rtt.as_nanos() as f64) / E6,
    }
  }
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  let to_string = match cli.format {
    Format::Human => {
      |ping: Ping| format!("{:.6}: {:.6} ms", ping.timestamp, ping.rtt)
    }
    Format::Csv => |ping: Ping| format!("{:.6},{}", ping.timestamp, ping.rtt),

    #[cfg(feature = "json")]
    Format::Json => {
      |ping: Ping| serde_json::to_string(&ping).unwrap_or_default()
    }
  };

  loop {
    match surge_ping::ping(cli.server, PAYLOAD).await {
      Ok((_, rtt)) => println!("{}", to_string(Ping::new(rtt))),
      Err(err) => eprintln!("error: {err}"),
    };

    tokio::time::sleep(Duration::from_millis(cli.interval)).await;
  }
}
