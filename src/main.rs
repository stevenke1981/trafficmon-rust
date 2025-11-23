use serde::Serialize;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

const NFT_TABLE: &str = "trafficmon";
const JSON_OUTPUT: &str = "/var/run/trafficmon.json";

#[derive(Serialize)]
struct TrafficData {
    eth1: u64,
    pppoe_wan: u64,
}

async fn nft_cmd(args: &[&str]) {
    let _ = Command::new("nft")
        .args(args)
        .output()
        .await;
}

async fn setup_nft() {
    nft_cmd(&["add", "table", "ip", NFT_TABLE]).await;
    nft_cmd(&["add", "chain", "ip", NFT_TABLE, "monitor", "{ type filter hook prerouting priority 0; }"]).await;

    nft_cmd(&["add", "counter", "ip", NFT_TABLE, "eth1_counter"]).await;
    nft_cmd(&["add", "counter", "ip", NFT_TABLE, "pppoe_counter"]).await;

    nft_cmd(&["add", "rule", "ip", NFT_TABLE, "monitor",
              "iifname", "eth1", "counter", "name", "eth1_counter"]).await;

    nft_cmd(&["add", "rule", "ip", NFT_TABLE, "monitor",
              "iifname", "pppoe-wan", "counter", "name", "pppoe_counter"]).await;

    nft_cmd(&["reset", "counters", "table", "ip", NFT_TABLE]).await;
}

async fn read_counter(counter: &str) -> u64 {
    let output = Command::new("nft")
        .args(["list", "counter", "ip", NFT_TABLE, counter])
        .output()
        .await
        .unwrap();

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("packets") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                return parts[3].parse::<u64>().unwrap_or(0);
            }
        }
    }
    0
}

#[tokio::main]
async fn main() {
    setup_nft().await;

    loop {
        let eth1 = read_counter("eth1_counter").await;
        let pppoe = read_counter("pppoe_counter").await;

        let data = TrafficData {
            eth1,
            pppoe_wan: pppoe,
        };

        let json = serde_json::to_string_pretty(&data).unwrap();
        tokio::fs::write(JSON_OUTPUT, json).await.unwrap();

        sleep(Duration::from_secs(5)).await;
    }
}
