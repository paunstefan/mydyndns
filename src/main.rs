use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{event, Level};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DnsData {
    data: String,
    name: String,
    ttl: i32,
    r#type: String,
}

const IP_URL: &str = "https://api.ipify.org?format=json";
const GD_URL: &str = "https://api.godaddy.com/v1/domains/paunstefan.xyz/records/A/home";
const API_KEY: &str = "sso-key [YOUR API KEY HERE]";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "mydyndns=trace")
    }
    let file_appender = tracing_appender::rolling::never("/var/log/", "mydyndns.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();

    event!(Level::INFO, "Checking address");

    match run_dyndns().await {
        Ok(_) => Ok(()),
        Err(e) => {
            event!(Level::ERROR, "{}", e);
            Err(e)
        }
    }
}

async fn run_dyndns() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let myip = client
        .get(IP_URL)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let gdresp = client
        .get(GD_URL)
        .header(AUTHORIZATION, API_KEY)
        .send()
        .await?
        .json::<Vec<DnsData>>()
        .await?;

    let mut dnsdata = vec![gdresp[0].clone()];

    if &dnsdata[0].data != myip.get("ip").ok_or("Hashmap has no 'ip' field")? {
        event!(
            Level::INFO,
            "Address changed to: {}",
            myip.get("ip").unwrap()
        );

        dnsdata[0].data = myip.get("ip").unwrap().clone();

        let _res = client
            .put(GD_URL)
            .header(AUTHORIZATION, API_KEY)
            .json(&dnsdata)
            .send()
            .await?;
    }

    Ok(())
}
