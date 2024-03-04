// dns_record.rs
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct DnsRecord {
    #[serde(rename = "clusterIP")] 
    pub cluster_ip: String,
    #[serde(rename = "clusterName")] 
    pub cluster_name: String,
    pub controller: String,
    pub fqdn: String,
    pub ip: String,
    pub kind: String,
    pub namespace: String,
    pub port: String,
}

impl DnsRecord {
    pub fn to_string(&self) -> String {
        format!(
            "Cluster IP: {}\nCluster Name: {}\nController: {}\nFQDN: {}\nIP: {}\nKind: {}\nNamespace: {}\nPort: {}\n",
            self.cluster_ip, self.cluster_name, self.controller, self.fqdn, self.ip, self.kind, self.namespace, self.port
        )
    }
}



/// Return the DNS A record representation of a DnsRecord
/// 
/// # Arguments
/// * `record` - A DnsRecord
/// 
/// # Returns
/// * `String` - A string representation of the A record
///
pub fn to_a_record(record: &DnsRecord) -> String {
    format!("{} IN A {}", record.fqdn, record.ip)
}

// Test the to_a_record function

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_a_record() {
        let record = DnsRecord {
            cluster_ip: "10.0.10.1".to_string(),
            cluster_name: "cluster.local".to_string(),
            controller: "service".to_string(),
            fqdn: "test.example.com".to_string(),
            ip: "192.168.178.101".to_string(),
            kind: "Service".to_string(),
            namespace: "default".to_string(),
            port: "80".to_string(),
        };
        let a_record = to_a_record(&record);
        assert_eq!(a_record, "test.example.com IN A 192.168.178.101");

    }

}

