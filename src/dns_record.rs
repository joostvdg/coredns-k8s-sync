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
    pub fn set_a_record(&mut self, domain_name: &str) {
        let domain_to_strip = format!(".{}", domain_name);
        let a_record_name = self.fqdn.replace(domain_to_strip.as_str(), "");
        self.a_record = format!("{} IN A {}", a_record_name, self.ip);
    }
}

// Test the to_a_record function

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_a_record() {
        let mut record = DnsRecord {
            fqdn: "test.example.com".to_string(),
            a_record: "".to_string(),
            ip: "192.168.178.101".to_string(),
            ..Default::default()
        };
        record.set_a_record( "example.com");
        assert_eq!(record.a_record, "test IN A 192.168.178.101");
    }
}
