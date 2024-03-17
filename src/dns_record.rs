use std::fmt;
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
    #[serde(skip_deserializing)]
    pub a_record: String,
    pub ip: String,
    pub kind: String,
    pub namespace: String,
    pub port: String,
    #[serde(skip_deserializing)]
    pub is_duplicate: bool,
}

impl Default for DnsRecord {
    fn default() -> Self {
        DnsRecord {
            cluster_ip: "".to_string(),
            cluster_name: "".to_string(),
            controller: "".to_string(),
            fqdn: "".to_string(),
            a_record: "".to_string(),
            ip: "".to_string(),
            kind: "".to_string(),
            namespace: "".to_string(),
            port: "".to_string(),
            is_duplicate: false,
        }
    }
}

impl fmt::Display for DnsRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Cluster IP: {}\nCluster Name: {}\nController: {}\nFQDN: {}\nIP: {}\nKind: {}\nNamespace: {}\nPort: {}\n",
            self.cluster_ip, self.cluster_name, self.controller, self.fqdn, self.ip, self.kind, self.namespace, self.port
        )
    }
}

impl DnsRecord {
    pub fn set_a_record(&mut self, domain_name: &str, padding_length: usize) {
        let domain_to_strip = format!(".{}", domain_name);
        let mut a_record_name = self.fqdn.replace(domain_to_strip.as_str(), "");
        for _ in 0..(padding_length - a_record_name.len()) {
            a_record_name.push(' ');
        }
        self.a_record = format!("{} IN A {}", a_record_name, self.ip);
    }
}

// Test the to_a_record function

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_a_record() {
        let padding_length = 10;

        let mut record_a = DnsRecord {
            fqdn: "test1.example.com".to_string(),
            a_record: "".to_string(),
            ip: "192.168.178.101".to_string(),
            ..Default::default()
        };
        record_a.set_a_record("example.com", padding_length);

        let mut record_b = DnsRecord {
            fqdn: "test2.example.com".to_string(),
            a_record: "".to_string(),
            ip: "192.168.178.102".to_string(),
            ..Default::default()
        };
        record_b.set_a_record("example.com", padding_length);

        assert_eq!(record_a.a_record, "test1      IN A 192.168.178.101");
        assert_eq!(record_b.a_record, "test2      IN A 192.168.178.102");
    }
}
