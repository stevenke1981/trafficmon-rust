// build.rs - 簡化版本
fn main() {
    // 只連結數學庫，不強制靜態
    println!("cargo:rustc-link-lib=m");
}