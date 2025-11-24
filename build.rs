// build.rs
fn main() {
    // 強制連結數學庫
    println!("cargo:rustc-link-lib=m");
    
    // 針對 musl 目標的設定
    let target = std::env::var("TARGET").unwrap();
    if target.contains("musl") {
        println!("cargo:rustc-link-lib=m");
        println!("cargo:rustc-link-arg=-static");
    }
}