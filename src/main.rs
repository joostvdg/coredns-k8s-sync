// main.rs
mod config;
mod dns_record;
mod dns_record_collector;
mod file_writer;

use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = config::load_config()?;
    info!("Config:\n{}", config.to_string());

    let mut collector = dns_record_collector::DnsRecordCollector::new(config.clone());
    collector.collect_dns_records().await?;

    let mut source_file_paths: Vec<String> = Vec::new();
    for source_path in config.source_file_paths.iter() {
        // validate the source file path
        if !std::path::Path::new(source_path).exists() {
            panic!("Source file path does not exist: {}", source_path);
        }
        source_file_paths.push(source_path.clone());
    }

    // TODO: configure intermediary result storage
    for source in collector.get_dns_records_by_source() {
        let dns_records = collector.get_dns_records_by_source().get(source.0.as_str()).unwrap();
        
        let local_test_records_file_path = "testdata/".to_owned() + source.0.as_str() + ".txt";
        file_writer::write_dns_records_to_file(
            dns_records.clone().as_mut_slice(),
            local_test_records_file_path.as_str(),
            source.0.as_str(),
        )
        .await?;
        source_file_paths.push(local_test_records_file_path.clone());
    }

    // TODO: read destination file path from config, and validate we can write to it
    let destination_file_path = "./examples/destination.home.lab";
    file_writer::merge_source_files(source_file_paths, destination_file_path).await?;

    // TODO: how do we reload the CoreDNS service?

    Ok(())
}
