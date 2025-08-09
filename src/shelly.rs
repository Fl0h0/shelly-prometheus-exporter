use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Meters {
    pub power: f64,
    pub total: f64,
    pub is_valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct Relays {
    pub ison: bool,
}

#[derive(Debug, Deserialize)]
pub struct Shelly {
    pub uptime: u64,
    pub relays: Vec<Relays>,
    pub meters: Vec<Meters>,

    pub ram_total: u64,
    pub ram_free: u64,
    pub fs_size: u64,
    pub fs_free: u64,
}

pub async fn scrape_shelly_plug(target: String) -> Result<Shelly, Box<dyn std::error::Error>> {
    let resp = Client::new()
        .get(format!("http://{target}/status"))
        .send()
        .await?;
    let shelly = resp.json::<Shelly>().await?;
    Ok(shelly)
}
