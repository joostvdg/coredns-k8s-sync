use crate::dns_record::DnsRecord;

// main.rs
mod config;
mod file_writer;
mod dns_record;
mod dns_record_collector;

use log::{info};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = config::load_config()?;
    info!("Config:\n{}", config.to_string());
    let mut collector = dns_record_collector::DnsRecordCollector::new(config);
    collector.collect_dns_records().await?;

    let source_file_paths = ["./examples/source.home.lab"];
    let destination_file_path = "./examples/destination.home.lab";
    file_writer::merge_source_files(&source_file_paths, &destination_file_path).await?;

    Ok(())
}