use serde::Serialize;
use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use tokio::time::{sleep, Duration};
use tokio::signal;

fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn get_interfaces() -> Vec<String> {
    let ifaces_str = get_env_or_default("TRAFFICMON_INTERFACES", "eth1 pppoe-wan");
    ifaces_str.split_whitespace().map(|s| s.to_string()).collect()
}

fn get_interval() -> u64 {
    get_env_or_default("TRAFFICMON_INTERVAL", "5")
        .parse()
        .unwrap_or(5)
}

fn get_output_path() -> String {
    get_env_or_default("TRAFFICMON_OUTPUT", "/var/run/trafficmon.json")
}

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

fn collect_traffic_data(interfaces: &[String]) -> TrafficData {
    let mut result = TrafficData {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        data: vec![],
    };

    for iface in interfaces {
        let cname = format!("cnt_{}", iface);
        match read_nft_counter(&cname) {
            Ok((packets, bytes)) => {
                result.data.push(IfaceData {
                    iface: iface.clone(),
                    packets,
                    bytes,
                });
            }
            Err(e) => {
                eprintln!("Error reading counter for {}: {}", iface, e);
                result.data.push(IfaceData {
                    iface: iface.clone(),
                    packets: 0,
                    bytes: 0,
                });
            }
        }
    }

    result
}

fn write_traffic_data(data: &TrafficData, output_path: &str) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)
        .map_err(|e| format!("Failed to open {}: {}", output_path, e))?;

    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write data: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Failed to sync file: {}", e))?;

    Ok(())
}

async fn monitor_loop() {
    let interfaces = get_interfaces();
    let interval = get_interval();
    let output_path = get_output_path();
    
    println!("Starting traffic monitor...");
    println!("Monitoring interfaces: {:?}", interfaces);
    println!("Output file: {}", output_path);
    println!("Update interval: {} seconds", interval);

    if let Err(e) = reset_nft() {
        eprintln!("Warning: Failed to reset counters: {}", e);
    } else {
        println!("Counters reset successfully");
    }

    loop {
        let data = collect_traffic_data(&interfaces);

        match write_traffic_data(&data, &output_path) {
            Ok(_) => {
                println!("Updated traffic data at timestamp {}", data.timestamp);
            }
            Err(e) => {
                eprintln!("Error writing traffic data: {}", e);
            }
        }

        sleep(Duration::from_secs(interval)).await;
    }
}

#[tokio::main]
async fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {:?}", panic_info);
    }));

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