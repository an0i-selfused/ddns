use reqwest::blocking::Client;
use std::{
    env::var,
    fs,
    net::{IpAddr, Ipv4Addr},
};

enum Success {
    Eq,
    Ne,
}

fn update() -> Result<(Success, Ipv4Addr), Box<dyn std::error::Error>> {
    let previous_ip: Option<Ipv4Addr> = fs::read_to_string("previous_ip.txt")
        .map(|s| s.parse().unwrap())
        .ok();

    let current_ip: Ipv4Addr = {
        let current_ip_api = var("DDNS_IP_API").unwrap_or("https://ip.anqi.fun".to_string());
        let client = Client::builder()
            .local_address(IpAddr::from([0, 0, 0, 0]))
            .build()?;
        client.get(current_ip_api).send()?.text()?.parse()?
    };

    if previous_ip.is_some() && previous_ip.unwrap() == current_ip {
        return Ok((Success::Eq, current_ip));
    };

    {
        let zone_id = var("CF_ZONE_ID")?;
        let dns_record_id = var("CF_DNS_RECORD_ID")?;
        let key = var("CF_KEY")?;
        let dns_api = format!(
            "https://api.cloudflare.com/client/v4/zones/{zone_id}/dns_records/{dns_record_id}"
        );
        let client = Client::new();
        client
            .patch(dns_api)
            .bearer_auth(key)
            .header("Content-Type", "application/json")
            .body(format!("{{\"content\":\"{current_ip}\"}}"))
            .send()?
            .error_for_status()?
    };

    fs::write("previous_ip.txt", current_ip.to_string())?;

    Ok((Success::Ne, current_ip))
}

fn main() {
    let secs_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (success, ip) = update().unwrap();

    match success {
        Success::Eq => println!("[{}] ok(eq): {}", secs_since_epoch, ip.to_string()),
        Success::Ne => println!("[{}] ok(ne): {}", secs_since_epoch, ip.to_string()),
    };
}
