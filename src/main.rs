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
    // Print the DNS records
    // TODO: add method to collector to return the records

    let source_file_paths = ["./examples/source.home.lab"];

    for source in collector.get_dns_records_by_source() {
        let local_test_records_file_path = "testdata/".to_owned() + source.0.as_str() + ".txt";
        file_writer::write_dns_records_to_file(source.1.as_slice(),
                                               local_test_records_file_path.as_str(),
                                               0.to_string().as_str()
        ).await?;
        source_file_paths.push(local_test_records_file_path);
    }

    let destination_file_path = "./examples/destination.home.lab";
    file_writer::merge_source_files(&source_file_paths, &destination_file_path).await?;

    Ok(())
}