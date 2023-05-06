pub fn get_player_id() -> u64 {
    #[cfg(target_arch = "wasm32")] // TODO fix rust analyzer
    let id: u64 = wasm_cookies::get("session_id")
        .unwrap_or_else(|| {
            log::info!("Did not get cookies!");
            Ok(String::new())
        })
        .unwrap()
        .parse()
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let id: u64 = 0;

    id
}
