// main.rs
mod config;
mod dns_record;
mod dns_record_collector;
mod file_writer;
mod file_watcher;

use std::collections::HashMap;

use dns_record::DnsRecord;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::{sync::mpsc, time::{sleep, Duration}};


use crate::dns_record_collector::RealDnsRecordFetcher;

use log::{error, info};

extern crate getopts;
use getopts::Options;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let mut opts = Options::new();
    opts.optopt("c", "config", "Set the config file path", "FILE");
    let matches = opts.parse(std::env::args().skip(1))?;

    let config_path = match matches.opt_str("c") {
        Some(path) => path,
        None => {
            eprintln!("Please provide a config file path with -c or --config");
            std::process::exit(1);
        }
    };



    let config_path_clone = config_path.clone();
    let config = config::load_config(config_path)?;
    info!("Config:\n{}", config.to_string());

    let mut source_file_paths: Vec<String> = Vec::new();
    for source_path in config.source_file_paths.iter() {
        // validate the source file path
        if !std::path::Path::new(source_path).exists() {
            panic!("Source file path does not exist: {}", source_path);
        }
        source_file_paths.push(source_path.clone());
    }


    // TODO: finish this
    // https://github.com/notify-rs/notify/issues/380
    // https://github.com/notify-rs/notify/blob/main/examples/async_monitor.rs
    // https://tokio.rs/tokio/tutorial/channels
    // Also needs to make the actual file writing loop async
    let (tx, mut rx) = mpsc::channel(100);
    let mut watcher = RecommendedWatcher::new(move |result: std::result::Result<notify::Event, notify::Error>| {
            tx.blocking_send(result).expect("Failed to send event");
        },
        notify::Config::default()
    )?;

    let mut paths = source_file_paths.clone();
    paths.push(config_path_clone);
    for path_string in paths {
        let path = std::path::PathBuf::from(path_string);
        watcher.watch(&path, RecursiveMode::NonRecursive).unwrap();
    }

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    while let Some(res) = rx.recv().await {
        tokio::spawn(async move {println!("got = {:?}", res);});
    }
    

    // handle config updates
    info!("Setup file watcher...");
    config::handle_config_update(rx).await;
    


    let period_time_in_minutes = Duration::from_secs(config.call_frequency_in_minutes * 60);
    // TODO:  Run the infinite loop in a separate task
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
