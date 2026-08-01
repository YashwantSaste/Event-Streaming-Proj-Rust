use std::collections::HashMap;
use std::time::Duration;

use producer::producer_client::ProducerClient;
use producer::producer_configuration::ProducerConfiguration;
use producer::retry_policy::RetryPolicy;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_options();
    let command = options
        .get("command")
        .map(String::as_str)
        .unwrap_or("produce");
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
    let key = options.get("key").map(|value| value.as_bytes().to_vec());

    let configuration = ProducerConfiguration::new(host, port, 1_048_576)?;
    let retry_policy = RetryPolicy::new(3, Duration::from_millis(100));
    let mut client = ProducerClient::new(configuration, retry_policy);

    match command {
        "create-topic" => {
            let topic = required_option(&options, "topic")?;
            let partitions = parse_u32(
                options.get("partitions").map(String::as_str).unwrap_or("1"),
                "partitions",
            )?;
            let segment_max_bytes = parse_u64(
                options
                    .get("segment-max-bytes")
                    .map(String::as_str)
                    .unwrap_or("1048576"),
                "segment-max-bytes",
            )?;
            let response = client
                .create_topic(topic, partitions, segment_max_bytes)
                .await?;
            println!("Created topic {}", response.topic());
        }
        "list-topics" => {
            for topic in client.list_topics().await? {
                println!("{topic}");
            }
        }
        "produce" => {
            let topic = required_option(&options, "topic")?;
            let message = required_option(&options, "message")?;
            let response = client
                .send(topic, partition, key, message.as_bytes().to_vec())
                .await?;
            println!(
                "Produced to {}:{} at offset {}",
                response.partition().topic(),
                response.partition().partition_id(),
                response.offset()
            );
        }
        other => return Err(format!("Unsupported --command {other}").into()),
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
