use crate::config::{Config, ExternalSource};
use crate::dns_record::DnsRecord;
use base64::{engine::general_purpose, Engine as _};
use log::{info, warn};
use reqwest::{Certificate, Url};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use async_trait::async_trait;

#[async_trait]
pub trait DnsRecordFetcher {
    async fn fetch_dns_records(&self, source: &ExternalSource, ca_cert_base64: &str) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>>;
}

pub struct RealDnsRecordFetcher;

#[async_trait]
impl DnsRecordFetcher for RealDnsRecordFetcher {
    async fn fetch_dns_records(&self, source: &ExternalSource, ca_cert_base64: &str) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
        let cert_as_decoded_bytes = general_purpose::STANDARD
            .decode(ca_cert_base64)
            .unwrap();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .add_root_certificate(Certificate::from_pem(cert_as_decoded_bytes.as_slice())?)
            .build()?;

        let result = client
            .get(Url::parse(&source.url).unwrap())
            .send()
            .await?
            .json()
            .await?;

        Ok(result)
    
    }
}

pub struct DnsRecordCollector {
    config: Config,
    dns_records_by_source: HashMap<String, Vec<DnsRecord>>,
    fetcher: Box<dyn DnsRecordFetcher>,
}

impl fmt::Display for DnsRecordCollector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Config: {}\nDNS Records by Source: {:?}\n",
            self.config, self.dns_records_by_source,
        )
    }
}

// TODO: make fetcher optional, so we use the real fetcher by default
impl DnsRecordCollector {
    pub fn new(config: Config, fetcher:  Box<dyn DnsRecordFetcher>   ) -> DnsRecordCollector {
        DnsRecordCollector {
            config,
            dns_records_by_source: HashMap::new(),
            fetcher: fetcher,
        }
    }

    // For each external_source in the config, fetch the DNS records and store them in the dns_records_by_source HashMap
    pub async fn collect_dns_records(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut fqds_seen = HashMap::new();

        for external_source in &self.config.external_sources {
            info!("Fetching DNS records from {}", external_source.source_name);
            
            let fetch_result = self.fetcher.fetch_dns_records(external_source, self.config.ca_cert_base64.as_str()).await;
            if fetch_result.is_err() {
                warn!(
                    "Failed to fetch DNS records from {} - {}",
                    external_source.source_name,
                    fetch_result.err().unwrap()
                );
                continue;
            }

            let mut dns_records = fetch_result.unwrap();
            dns_records.sort_by_key( |record| record.fqdn.clone());
            let mut longest_name = 0;

            for record in &mut dns_records {
                let name_length = record.fqdn.len() - external_source.domain_name.len();
                if name_length > longest_name {
                    longest_name = name_length;
                }
            }

            let padding_length = longest_name + 4;
            for record in &mut dns_records {
                record.set_a_record(external_source.domain_name.as_str(), padding_length);
                if fqds_seen.contains_key(&record.fqdn) {
                    record.is_duplicate = true;
                } else {
                    fqds_seen.insert(record.fqdn.clone(), true);
                }
            }

            self.dns_records_by_source
                .insert(external_source.source_name.clone(), dns_records);
            info!(
                "Fetched {} DNS records from {}",
                self.dns_records_by_source[&external_source.source_name].len(),
                external_source.source_name
            );
        }
        
        

        // TODO: deduplicate the DNS records
        Ok(())
    }

    /// Returns the collected DNS records by source
    ///
    /// # Returns
    /// * `HashMap<String, Vec<DnsRecord>>` - A HashMap containing the DNS records by source
    ///
    pub fn get_dns_records_by_source(&self) -> &HashMap<String, Vec<DnsRecord>> {
        &self.dns_records_by_source
    }

    // TODO: implement the merge_dns_records function where we store the canonical DNS records
    // TODO: sort the results by FQDN
    // TODO: strip the domain name from the FQDN
}



#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    pub struct MockDnsRecordFetcher;

    #[async_trait]
    impl DnsRecordFetcher for MockDnsRecordFetcher {
        async fn fetch_dns_records(&self, _source: &ExternalSource, _ca_cert_base64: &str) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
            let records = vec![
                    DnsRecord { fqdn: "test1.example.com".to_string(), ..Default::default() },
                    DnsRecord { fqdn: "test1.example.com".to_string(), ..Default::default() },
                    DnsRecord { fqdn: "test2.example.com".to_string(), ..Default::default() },
                ];
            
            Ok(records)
        }
    }

    #[tokio::test]
    async fn test_collect_dns_records_marks_duplicates() {
        let config = Config {
            external_sources: vec![ExternalSource {
                url: "https://example.com".to_string(),
                domain_name: "example.com".to_string(),
                source_name: "test".to_string(),
            }],
            temp_storage_path: "temp.home.lab".to_string(),
            destination_file_path: "destination.home.lab".to_string(),
            source_file_paths: vec!["source1.home.lab".to_string()],
            ttl: 0,
            call_frequency: 0,
            ca_cert_base64: "test".to_string(),
            log_level: "info".to_string(),
        };
        let mut collector = DnsRecordCollector::new(config, Box::new(MockDnsRecordFetcher));

        collector.collect_dns_records().await.unwrap();
        let records: Vec<DnsRecord> = collector.dns_records_by_source.get("test").unwrap().to_vec();
        assert!(!records[0].is_duplicate);
        assert!(records[1].is_duplicate);
        assert!(!records[2].is_duplicate);
    }
}