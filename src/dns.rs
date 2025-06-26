use crate::models::RedirectConfiguration;
use hickory_client::client::{Client, ClientHandle};
use hickory_client::proto::rr::{DNSClass, Name, Record, RecordType};
use hickory_client::proto::runtime::TokioRuntimeProvider;
use hickory_client::proto::udp::UdpClientStream;
use hickory_client::proto::xfer::DnsResponse;
use std::net::SocketAddr;
use std::str::FromStr;

async fn get_txt_records(domain: &str) -> Result<Vec<String>, String> {
    let address = SocketAddr::from(([8, 8, 8, 8], 53));
    let conn = UdpClientStream::builder(address, TokioRuntimeProvider::default()).build();
    let (mut client, bg) = Client::connect(conn).await?;
    tokio::spawn(bg);

    let name = Name::from_str(format!("{}.", domain).as_str())?;

    let response: DnsResponse = client
        .query(name, DNSClass::IN, RecordType::TXT)
        .await
        .unwrap();

    let answers: &[Record] = response.answers();

    let txt_refs = answers
        .iter()
        .flat_map(|record| record.data().as_txt())
        .collect::<Vec<_>>();

    Ok(txt_refs
        .iter()
        .map(|txt| txt.to_string()) // Convert each &TXT to String
        .collect())
}

async fn get_goeie_records(domain: &str) -> Result<Vec<String>, String> {
    let records = get_txt_records(domain).await?;
    Ok(records
        .into_iter()
        .filter(|record| record.starts_with("goeie-"))
        .collect())
}

pub async fn get_redirect_config(domain: &str) -> Result<RedirectConfiguration, String> {
    let records = get_goeie_records(domain).await?;
    let target = records
        .iter()
        .find(|record| record.starts_with("goeie-redirect-to="));

    if target.is_none() {
        return Err("Failed to find redirect target".to_string());
    }

    let target = target
        .unwrap()
        .to_string()
        .split_once("=")
        .iter()
        .next()
        .unwrap()
        .1
        .to_string();

    Ok(RedirectConfiguration {
        redirect_target_url: target,
    })
}
