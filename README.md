# coredns-k8s-sync

Synchronize CoreDNS config file with DNS entries from Kubernetes resources

## TODO

* [ ] Read in a source file containing DNS records from Kubernetes resources
    * [ ] Collect this source file from IDEC endpoints
      * [ ] Configure these endpoints with a configuration file
    * [ ] Detect duplications and remove them (and log them)
    * [ ] Sort the records in each section, so that output is consistent
    * [ ] Merge the records into a single file with source files
* [ ] Integration test, using several test source files and then verifying the output
