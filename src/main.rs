use serde::Serialize;
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::time::{sleep, Duration};
use tokio::signal;

const OUTPUT_PATH: &str = "/var/run/trafficmon.json";
const INTERFACES: &[&str] = &["eth1", "pppoe-wan"];
const UPDATE_INTERVAL_SECS: u64 = 5;

#[derive(Serialize)]
struct IfaceData {
    iface: String,
    packets: u64,
    bytes: u64,
}

#[derive(Serialize)]
struct TrafficData {
    timestamp: u64,
    data: Vec<IfaceData>,
}

/// 從 nftables counter 讀取封包數和位元組數
fn read_nft_counter(name: &str) -> Result<(u64, u64), String> {
    let output = Command::new("nft")
        .args(["list", "counter", "inet", "trafficmon", name])
        .output()
        .map_err(|e| format!("Failed to execute nft: {}", e))?;

    if !output.status.success() {
        return Err(format!("nft command failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    let txt = String::from_utf8_lossy(&output.stdout);
    let mut packets = 0u64;
    let mut bytes = 0u64;

    // 解析 nftables 輸出: "counter packets X bytes Y"
    for line in txt.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        for i in 0..words.len() {
            if words[i] == "packets" && i + 1 < words.len() {
                packets = words[i + 1].parse().unwrap_or(0);
            }
            if words[i] == "bytes" && i + 1 < words.len() {
                bytes = words[i + 1].parse().unwrap_or(0);
            }
        }
    }

    Ok((packets, bytes))
}

/// 重置所有 nftables counters
fn reset_nft() -> Result<(), String> {
    let output = Command::new("nft")
        .args(["reset", "counters", "inet", "trafficmon"])
        .output()
        .map_err(|e| format!("Failed to reset counters: {}", e))?;

    if !output.status.success() {
        return Err(format!("Reset failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

/// 收集所有介面的流量資料
fn collect_traffic_data() -> TrafficData {
    let mut result = TrafficData {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        data: vec![],
    };

    for iface in INTERFACES {
        let cname = format!("cnt_{}", iface);
        match read_nft_counter(&cname) {
            Ok((packets, bytes)) => {
                result.data.push(IfaceData {
                    iface: iface.to_string(),
                    packets,
                    bytes,
                });
            }
            Err(e) => {
                eprintln!("Error reading counter for {}: {}", iface, e);
                // 失敗時仍然加入資料，但數值為 0
                result.data.push(IfaceData {
                    iface: iface.to_string(),
                    packets: 0,
                    bytes: 0,
                });
            }
        }
    }

    result
}

/// 將流量資料寫入 JSON 檔案
fn write_traffic_data(data: &TrafficData) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(OUTPUT_PATH)
        .map_err(|e| format!("Failed to open {}: {}", OUTPUT_PATH, e))?;

    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write data: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Failed to sync file: {}", e))?;

    Ok(())
}

/// 主監控迴圈
async fn monitor_loop() {
    println!("Starting traffic monitor...");
    println!("Monitoring interfaces: {:?}", INTERFACES);
    println!("Output file: {}", OUTPUT_PATH);
    println!("Update interval: {} seconds", UPDATE_INTERVAL_SECS);

    // 初始重置計數器
    if let Err(e) = reset_nft() {
        eprintln!("Warning: Failed to reset counters: {}", e);
    } else {
        println!("Counters reset successfully");
    }

    loop {
        let data = collect_traffic_data();

        match write_traffic_data(&data) {
            Ok(_) => {
                println!("Updated traffic data at timestamp {}", data.timestamp);
            }
            Err(e) => {
                eprintln!("Error writing traffic data: {}", e);
            }
        }

        sleep(Duration::from_secs(UPDATE_INTERVAL_SECS)).await;
    }
}

#[tokio::main]
async fn main() {
    // 設定 panic hook 以便更好地除錯
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {:?}", panic_info);
    }));

    // 使用 tokio::select! 來處理信號
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        }
        _ = monitor_loop() => {
            eprintln!("Monitor loop ended unexpectedly");
        }
    }

    println!("Traffic monitor stopped");
}
