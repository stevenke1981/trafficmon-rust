use serde::Serialize;
use std::process::Command;
use std::fs::File;
use std::io::Write;
use tokio::time::{sleep, Duration};

const OUTPUT_PATH: &str = "/var/run/trafficmon.json";
const INTERFACES: &[&str] = &["eth1", "pppoe-wan"];

#[derive(Serialize)]
struct IfaceData {
    iface: String,
    rx: u64,
    tx: u64,
}

#[derive(Serialize)]
struct TrafficData {
    data: Vec<IfaceData>,
}

fn read_nft_counter(name: &str) -> (u64, u64) {
    let output = Command::new("nft")
        .args(["list", "counter", "inet", "trafficmon", name])
        .output();

    if let Ok(out) = output {
        let txt = String::from_utf8_lossy(&out.stdout);

        let rx = txt
            .lines()
            .find(|l| l.contains("packets"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(0);

        let tx = txt
            .lines()
            .find(|l| l.contains("bytes"))
            .and_then(|line| {
                line.split_whitespace()
                    .nth(1)
                    .unwrap_or("0")
                    .parse::<u64>()
                    .ok()
            })
            .unwrap_or(0);

        (rx, tx)
    } else {
        (0, 0)
    }
}

fn reset_nft() {
    let _ = Command::new("nft")
        .args(["reset", "counters", "inet", "trafficmon"])
        .output();
}

#[tokio::main]
async fn main() {
    reset_nft();

    loop {
        let mut result = TrafficData { data: vec![] };

        for iface in INTERFACES {
            let cname = format!("cnt_{}", iface);
            let (rx, tx) = read_nft_counter(&cname);

            result.data.push(IfaceData {
                iface: iface.to_string(),
                rx,
                tx,
            });
        }

        if let Ok(mut f) = File::create(OUTPUT_PATH) {
            let _ = f.write_all(serde_json::to_string(&result).unwrap().as_bytes());
        }

        sleep(Duration::from_secs(5)).await;
    }
}
