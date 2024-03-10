use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Certificate, Url};
use crate::config::{Config, ExternalSource};
use crate::dns_record::DnsRecord;
use base64::{Engine as _, engine::{self, general_purpose}};
use log::{info, warn};

pub struct DnsRecordCollector {
    config: Config,
    dns_records_by_source: HashMap<String, Vec<DnsRecord>>,
    canonical_dns_records: Vec<DnsRecord>,
}

impl DnsRecordCollector {
    pub fn new(config: Config) -> DnsRecordCollector {
        DnsRecordCollector {
            config,
            dns_records_by_source: HashMap::new(),
            canonical_dns_records: Vec::new(),
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "Config: {}\nDNS Records by Source: {:?}\nCanonical DNS Records: {:?}\n",
            self.config.to_string(), self.dns_records_by_source, self.canonical_dns_records
        )
    }

    // For each external_source in the config, fetch the DNS records and store them in the dns_records_by_source HashMap
    pub async fn collect_dns_records(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for external_source in &self.config.external_sources {
            info!("Fetching DNS records from {}", external_source.source_name  );
            // TODO: handle possible faillure of collecting DNS records, which is OK, we log a warning and continue

            let fetch_result = self.fetch_dns_records(&external_source).await;
            if fetch_result.is_err() {
                warn!("Failed to fetch DNS records from {}", external_source.source_name);
                continue;
            }

            let dns_records = fetch_result.unwrap();
            self.dns_records_by_source.insert(external_source.source_name.clone(), dns_records);
            info!("Fetched {} DNS records from {}", self.dns_records_by_source[&external_source.source_name].len(), external_source.source_name);
        }
        Ok(())
    }

    // Fetch the DNS records from the external source
    async fn fetch_dns_records(&self, source: &ExternalSource) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
        let cert_as_decoded_bytes = general_purpose::STANDARD
            .decode(self.config.ca_cert_base64.as_str()).unwrap();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .add_root_certificate(Certificate::from_pem(cert_as_decoded_bytes.as_slice())?)
            .build()?;

        let result = client.get(Url::parse(&*source.url).unwrap())
            .send()
            .await?
            .json()
            .await?;

        Ok(result)
    }

    /// Returns the collected DNS records by source
    ///
    /// # Returns
    /// * `HashMap<String, Vec<DnsRecord>>` - A HashMap containing the DNS records by source
    ///
    pub fn get_dns_records_by_source(&self) -> &HashMap<String, Vec<DnsRecord>> {
        &self.dns_records_by_source
    }
}
