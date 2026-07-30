use std::collections::HashMap;

use consumer::consumer_client::ConsumerClient;
use consumer::consumer_configuration::ConsumerConfiguration;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options();
    let topic = required_option(&options, "topic")?;
    let group = options
        .get("group")
        .map(String::as_str)
        .unwrap_or("default");
    let host = options
        .get("host")
        .map(String::as_str)
        .unwrap_or("127.0.0.1");
    let port = parse_u16(
        options.get("port").map(String::as_str).unwrap_or("9092"),
        "port",
    )?;
    let partition = parse_u32(
        options.get("partition").map(String::as_str).unwrap_or("0"),
        "partition",
    )?;
    let offset = parse_u64(
        options.get("offset").map(String::as_str).unwrap_or("0"),
        "offset",
    )?;
    let max_records = parse_u32(
        options
            .get("max-records")
            .map(String::as_str)
            .unwrap_or("10"),
        "max-records",
    )?;

    let configuration = ConsumerConfiguration::new(host, port, group, 1_048_576)?;
    let mut client = ConsumerClient::new(configuration);
    client.subscribe(topic)?;
    let records = client.poll(partition, offset, max_records).await?;

    for record in records {
        println!(
            "{}:{}:{} {}",
            record.topic(),
            record.partition_id(),
            record.offset(),
            String::from_utf8_lossy(record.payload().bytes())
        );
    }
    Ok(())
}

fn parse_options() -> HashMap<String, String> {
    let mut args = std::env::args().skip(1);
    let mut options = HashMap::new();
    while let Some(argument) = args.next() {
        if let Some(key) = argument.strip_prefix("--") {
            if let Some(value) = args.next() {
                options.insert(key.to_string(), value);
            }
        }
    }
    options
}

fn required_option<'a>(
    options: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    options
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("Missing required option --{key}").into())
}

fn parse_u16(value: &str, key: &str) -> Result<u16, Box<dyn std::error::Error>> {
    value
        .parse::<u16>()
        .map_err(|error| format!("Invalid --{key}: {error}").into())
}

fn parse_u32(value: &str, key: &str) -> Result<u32, Box<dyn std::error::Error>> {
    value
        .parse::<u32>()
        .map_err(|error| format!("Invalid --{key}: {error}").into())
}

fn parse_u64(value: &str, key: &str) -> Result<u64, Box<dyn std::error::Error>> {
    value
        .parse::<u64>()
        .map_err(|error| format!("Invalid --{key}: {error}").into())
}
