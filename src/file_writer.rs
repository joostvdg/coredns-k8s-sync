use io::ErrorKind;
use std::io;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use ErrorKind::NotFound;

use crate::dns_record:: DnsRecord;
use log::{info, warn};

/// Merge the contents of the Source file with our own content
///
/// # Arguments
/// * `source_file_paths` - A slice of string containing the path to the source files, with the first being the primary source file (containing the SOA record)
/// * `destination_file_path` - A string containing the path to the destination file
///
/// # Returns
/// * `io::Result<()>` - A result indicating success or failure
///
pub async fn merge_source_files(
    source_file_paths: Vec<String>,
    destination_file_path: &str,
) -> io::Result<()> {
    info!(
        "Merging source files: {:?} into destination file: {}",
        source_file_paths, destination_file_path
    );

    // Verify the source files exists
    if source_file_paths.is_empty() {
        warn!("No source files provided");
        let error = io::Error::new(ErrorKind::InvalidInput, "No source file provided");
        let result = Err(error);
        return result;
    } else {
        info!("Source files: {:?}", source_file_paths);
    }

    // Merge the contents of the Source file with our own content
    // First, create a empty string
    let mut destination_file_content = String::new();
    // Add the delimiter comment
    destination_file_content.push_str("; This file was generated by the coredns-k8s-sync module\n");
    destination_file_content.push_str("; Do not edit this file manually\n");
    destination_file_content.push_str("; Original Source\n");

    // Add the source files content
    for source_file_path in source_file_paths {
        if !Path::new(&source_file_path).exists() {
            warn!("Source file {} not found", source_file_path);
            let error = io::Error::new(NotFound, "File not found");
            let result = Err(error);
            return result;
        } else {
            info!("Source file {} found", source_file_path);
            let content = read_content_from_source_file(&source_file_path).await?;
            destination_file_content.push('\n');
            destination_file_content.push_str("; Source File: ");
            destination_file_content.push_str(&source_file_path);
            destination_file_content.push('\n');
            destination_file_content.push_str(&content);
            destination_file_content.push('\n');
            destination_file_content.push('\n');
        }
    }

    info!("Writing to destination file: {}", destination_file_path);
    tokio::fs::write(destination_file_path, destination_file_content)
        .await
        .unwrap();

    Ok(())
}

async fn read_content_from_source_file(source_file_path: &str) -> io::Result<String> {
    info!("Opening source file: {}", source_file_path);
    let mut source_file = OpenOptions::new().read(true).open(source_file_path).await?;
    let mut source_file_content = String::new();
    info!("Reading source file: {}", source_file_path);
    source_file.read_to_string(&mut source_file_content).await?;
    Ok(source_file_content)
}

/// Write DNSRecords to a file, one record per line, one file per source
///
/// # Arguments
/// * `dns_records` - A slice of DnsRecord
/// * `destination_file_path` - A string containing the path to the destination file
/// * `source_name` - A string containing the name of the source
///
/// # Returns
/// * `io::Result<()>` - A result indicating success or failure
///
pub async fn write_dns_records_to_file(
    dns_records: & mut [DnsRecord],
    destination_file_path: &str,
    source_name: &str,
) -> io::Result<usize> {
    
    let mut destination_file_content = String::new();
    destination_file_content.push_str("; Source: ");
    destination_file_content.push_str(source_name);
    destination_file_content.push('\n');

    if dns_records.is_empty() {
        warn!("No DNS records to write to file: {}", destination_file_path);
        destination_file_content.push_str("; No DNS records found\n");
    }

    // Sort the records by FQDN
    dns_records.sort_by_key(|record| record.fqdn.clone());

    info!("Writing DNS records to file: {}", destination_file_path);
    let mut records_written = 0;
    for record in dns_records {
        if record.is_duplicate {
            warn!("Duplicate DNS record found: {}", record.a_record.as_str());
            destination_file_content.push_str("; ");
            destination_file_content.push_str(record.a_record.as_str());
            destination_file_content.push_str(" - Duplicate");
            destination_file_content.push('\n');
        } else {
            destination_file_content.push_str(record.a_record.as_str());
            destination_file_content.push('\n');
        }
        records_written += 1;
    }
    
    // Add a newline at the end of the file
    destination_file_content.push('\n');
    
    let file_write_result = tokio::fs::write(destination_file_path, destination_file_content).await;
    match file_write_result {
        Ok(_) => {
            info!(
                "Wrote {} DNS records to file: {}",
                records_written,
                destination_file_path
            );
            Ok(records_written)
        }
        Err(e) => {
            warn!("Failed to write DNS records to file: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use std::fs::read_to_string;

    const DOMAIN_NAME: &str = "example.com";
    const PADDING: usize = 5;

    #[tokio::test]
    async fn test_merge_source_files() {
        // Print current working directory
        let cwd = std::env::current_dir().unwrap();
        println!("The current directory is {}", cwd.display());

        let test_file1 = "examples/source.home.lab";
        let mut source_file1_path = String::new();
        source_file1_path.push_str(cwd.display().to_string().as_str());
        source_file1_path.push('/');
        source_file1_path.push_str(test_file1);
        println!("The source file is {}", source_file1_path);
        let file_name_1 = source_file1_path.clone();

        let test_file2 = "examples/source.mandarin.compose";
        let mut source_file2_path = String::new();
        source_file2_path.push_str(cwd.display().to_string().as_str());
        source_file2_path.push('/');
        source_file2_path.push_str(test_file2);
        println!("The source file is {}", source_file2_path);
        let file_name_2 = source_file2_path.clone();

        let destination_file_name = "testdata/db.home.lab";
        let mut destination_file_path = String::new();
        destination_file_path.push_str(cwd.display().to_string().as_str());
        destination_file_path.push('/');
        destination_file_path.push_str(destination_file_name);
        println!("The destination file is {}", destination_file_path);
        let test_file_1 = destination_file_path.clone();

        // Call the function to test
        let test_files = vec![file_name_1, file_name_2];
        let result = merge_source_files(test_files, &test_file_1).await;

        // Check the result
        assert!(
            result.is_ok(),
            "Failed to merge source files: {:?}",
            result.err()
        );

        // Check the content of the destination file
        let mut destination_content = String::new();
        let mut file = File::open(destination_file_path.clone()).await.unwrap();
        file.read_to_string(&mut destination_content).await.unwrap();
        println!("Destination file content: {}", destination_content);
        assert!(
            destination_content.contains("; Do not edit this file manually"),
            "Destination file content does not contain generated comment"
        );
        assert!(
            destination_content.contains("$ORIGIN home.lab."),
            "Destination file content does not contain content from source.home.lab"
        );
        assert!(
            destination_content.contains("portainer    IN A     192.168.178.123"),
            "Destination file content does not contain content from source.mandarin.compose"
        );

        // Clean up
        tokio::fs::remove_file(destination_file_path).await.unwrap();
    }

    async fn generate_test_dns_records() -> Vec<DnsRecord> {
        

        let mut dns_record_a = DnsRecord {
            fqdn: "a.example.com".to_string(),
            ip: "127.0.0.2".to_string(),
            ..Default::default()
        };
        dns_record_a.set_a_record(DOMAIN_NAME, PADDING);

        let mut  dns_record_b = DnsRecord {
            fqdn: "b.example.com".to_string(),
            ip: "127.0.0.1".to_string(),
            ..Default::default()
        };
        dns_record_b.set_a_record(DOMAIN_NAME, PADDING);

        let mut dns_record_c = DnsRecord {
            fqdn: "c.example.com".to_string(),
            ip: "127.0.0.3".to_string(),
            ..Default::default()
        };
        dns_record_c.set_a_record(DOMAIN_NAME, PADDING);

        let mut dns_record_d = DnsRecord {
            fqdn: "d.example.com".to_string(),
            ip: "127.0.0.4".to_string(),
            ..Default::default()
        };
        dns_record_d.set_a_record(DOMAIN_NAME, PADDING);

        let mut dns_record_e = DnsRecord {
            fqdn: "e.example.com".to_string(),
            ip: "127.0.0.4".to_string(),
            ..Default::default()
        };
        dns_record_e.set_a_record(DOMAIN_NAME, PADDING);
        let mut dns_record_e_dup = DnsRecord {
            fqdn: "e.example.com".to_string(),
            ip: "127.0.0.4".to_string(),
            ..Default::default()
        };
        dns_record_e_dup.set_a_record(DOMAIN_NAME, PADDING);
        dns_record_e_dup.is_duplicate = true;

        vec![dns_record_b, dns_record_d, dns_record_a, dns_record_c, dns_record_e, dns_record_e_dup]
    }

    #[tokio::test]
    async fn test_write_dns_records_to_file_success() {
        let mut dns_records = generate_test_dns_records().await;
        let destination_file_path = "testdata/test_write_dns_records_to_file_success";
        let source_name = "test_source";

        let result =
            write_dns_records_to_file(dns_records.as_mut_slice(), destination_file_path, source_name).await;
        assert!(
            result.is_ok(),
            "Failed to write DNS records to file: {:?}",
            result.err()
        );

        // Clean up
        tokio::fs::remove_file(destination_file_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_write_dns_records_to_file_invalid_path() {
        let mut dns_records = generate_test_dns_records().await;
        let destination_file_path = "/root/test_write_dns_records_to_file_invalid_path";
        let source_name = "test_source";

        let result =
            write_dns_records_to_file(dns_records.as_mut_slice(), destination_file_path, source_name).await;
        assert!(
            result.is_err(),
            "Expected an error due to invalid file path"
        );
    }

    // Test if the records are in order
    #[tokio::test]
    async fn test_write_dns_records_to_file_order() {
        let mut dns_records = generate_test_dns_records().await;
        let destination_file_path = "testdata/test_write_dns_records_to_file_order";
        let source_name = "test_source";

        let result =
            write_dns_records_to_file(dns_records.as_mut_slice(), destination_file_path, source_name).await;
        assert!(
            result.is_ok(),
            "Failed to write DNS records to file: {:?}",
            result.err()
        );

        let lines: Vec<_> = read_to_string(destination_file_path) 
            .unwrap()  // panic on possible file-reading errors
            .lines()  // split the string into an iterator of string slices
            .map(String::from)  // make each slice into a string
            .collect();  // gather them together into a vector

        println!("Destination file content: \n{:?}", lines);
        

        let expected_line_1 = "a     IN A 127.0.0.2";
        let expected_line_2 = "b     IN A 127.0.0.1";
        let expected_line_3 = "c     IN A 127.0.0.3";
        let expected_line_4 = "d     IN A 127.0.0.4";
        let expected_line_5 = "e     IN A 127.0.0.4";

        assert_eq!(lines[1], expected_line_1);
        assert_eq!(lines[2], expected_line_2);
        assert_eq!(lines[3], expected_line_3);
        assert_eq!(lines[4], expected_line_4);
        assert_eq!(lines[5], expected_line_5);
        
    }

    #[tokio::test]
    async fn test_write_dns_records_to_file_duplicates_are_commented_out() {
        let mut dns_records = generate_test_dns_records().await;
        let destination_file_path = "testdata/test_write_dns_records_to_file_order";
        let source_name = "test_source";

        let result =
            write_dns_records_to_file(dns_records.as_mut_slice(), destination_file_path, source_name).await;
        assert!(
            result.is_ok(),
            "Failed to write DNS records to file: {:?}",
            result.err()
        );

        let lines: Vec<_> = read_to_string(destination_file_path) 
            .unwrap()  // panic on possible file-reading errors
            .lines()  // split the string into an iterator of string slices
            .map(String::from)  // make each slice into a string
            .collect();  // gather them together into a vector

        println!("Destination file content: \n{:?}", lines);
        

        let expected_line_5 = "e     IN A 127.0.0.4";
        let expected_line_6 = "; e     IN A 127.0.0.4 - Duplicate";

        assert_eq!(lines[5], expected_line_5);
        assert_eq!(lines[6], expected_line_6);
        
    }

    #[tokio::test]
    async fn test_write_dns_records_to_file_empty_records() {
        let mut dns_records: Vec<DnsRecord> = Vec::new();
        let destination_file_path = "testdata/test_write_dns_records_to_file_empty_records";
        let source_name = "test_source";

        let result =
            write_dns_records_to_file(dns_records.as_mut_slice(), destination_file_path, source_name).await;
        assert!(
            result.is_ok(),
            "Failed to write DNS records to file: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap(), 0, "Expected 0 records to be written");

        let expected_line_one = "; Source: test_source\n";
        let expected_line_two = "; No DNS records found\n";

        let mut destination_content = String::new();
        let mut file = File::open(destination_file_path).await.unwrap();
        file.read_to_string(&mut destination_content).await.unwrap();
        println!("Destination file content: {}", destination_content);
        assert!(
            destination_content.contains(expected_line_one),
            "Destination file content does not contain expected line one"
        );
        assert!(
            destination_content.contains(expected_line_two),
            "Destination file content does not contain expected line two"
        );

        // Clean up
        tokio::fs::remove_file(destination_file_path).await.unwrap();
    }
}
