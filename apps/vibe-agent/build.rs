use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        // EasyTier pulls in pnet's WinPcap bindings, which otherwise make Packet.dll a
        // load-time dependency for the agent binary on Windows. Delay-loading keeps the
        // relay_polling path usable unless overlay networking is actually exercised.
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-arg=/DELAYLOAD:Packet.dll");
    }
}
