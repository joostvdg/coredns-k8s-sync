// main.rs
mod config;
mod coredns_handler;
mod dns_record;
mod dns_record_collector;
mod file_writer;

use std::collections::HashMap;

use dns_record::DnsRecord;
use tokio::time::{sleep, Duration};

use crate::dns_record_collector::RealDnsRecordFetcher;

use log::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = config::load_config()?;
    info!("Config:\n{}", config.to_string());

    let period_time_in_minutes = Duration::from_secs(config.call_frequency_in_minutes * 60);

    let mut source_file_paths: Vec<String> = Vec::new();
    for source_path in config.source_file_paths.iter() {
        // validate the source file path
        if !std::path::Path::new(source_path).exists() {
            panic!("Source file path does not exist: {}", source_path);
        }
        source_file_paths.push(source_path.clone());
    }

    // Run the infinite loop in a separate task
    // let loop_task = tokio::spawn(async move {
    loop {
        info!("Restarting CoreDNS update loop...");
        let mut collector = dns_record_collector::DnsRecordCollector::new(
            config.clone(),
            Box::new(RealDnsRecordFetcher),
        );
        let record_map = collector.collect_dns_records().await.unwrap();
        std::mem::drop(collector);

        let result = write_records(record_map, config.temp_storage_path.clone()).await;
        let additional_source_file_paths: Vec<String> = match result {
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed to write DNS records to file: {}", e);
                continue;
            }
        };

        // merge the source file paths
        let mut source_file_paths = source_file_paths.clone();
        source_file_paths.extend(additional_source_file_paths);

        let result = file_writer::merge_source_files(
            source_file_paths,
            &config.destination_file_path.clone(),
        )
        .await;
        match result {
            Ok(_) => {
                info!("Successfully merged source files")
            }
            Err(e) => {
                error!("Failed to merge source files: {}", e)
            }
        }

        reload_coredns_service().await.unwrap();
        sleep(period_time_in_minutes).await;
    }
}

pub async fn write_records(
    dns_records_by_source: HashMap<String, Vec<DnsRecord>>,
    temp_storage_path: String,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut source_file_paths: Vec<String> = Vec::new();
    for source in dns_records_by_source {
        let dns_records = source.1;

        let local_test_records_file_path = temp_storage_path.clone() + source.0.as_str() + ".txt";
        file_writer::write_dns_records_to_file(
            dns_records.clone().as_mut_slice(),
            local_test_records_file_path.as_str(),
            source.0.as_str(),
        )
        .await?;
        source_file_paths.push(local_test_records_file_path.clone());
    }
    Ok(source_file_paths)
}

pub async fn reload_coredns_service() -> Result<(), Box<dyn std::error::Error>> {
    info!("Reloading CoreDNS service...");
    let coredns_restart_result = coredns_handler::restart_coredns().await;
    match coredns_restart_result {
        Ok(_) => {
            info!("Successfully called CoreDNS service restart command")
        }
        Err(e) => {
            error!("Failed to restart CoreDNS service: {}", e)
        }
    }
    Ok(())
}
