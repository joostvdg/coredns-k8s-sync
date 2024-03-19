# coredns-k8s-sync

Synchronize CoreDNS config file with DNS entries from Kubernetes resources

## TODO

* [ ] Reload CoreDNS withouth using `systemctl` or `sudo`
    * https://coredns.io/plugins/reload/
* [X] Run as SystemD service
    * [ ] Configure the service to restart on file change
    * [ ] Configure the service to restart on signal
* [ ] Restart CoreDNS when the destination file changes
* [ ] Gracefull shutdown, when the program is terminated
    * [ ] Wait for the CoreDNS restart to complete
    * [ ] Ensure we stop or wait for the DNS Collector to finish before closing
    * Inspiration from [Tokio Graceful Shutdown](https://tokio.rs/tokio/tutorial/graceful_shutdown) docs
* [ ] Have proper retry logic for the DNS Collector
* [ ] Integration test, using several test source files and then verifying the output
* [ ] Support authentication for IDEC endpoints
* [ ] CI/CD workflow
* [ ] Publish as Ubuntu package
* [ ] Publush via Homebrew

## Rust Tools Required

```shell
rustup component add rustfmt
rustup component add clippy
cargo install cargo-audit
```

## Testing With Kind

* https://istio.io/latest/docs/setup/platform-setup/kind/

```shell
kind create cluster --name coredns-k8s-sync
```

```shell
kubectl config use-context kind-coredns-k8s-sync
```

```shell
kubectl apply -f https://raw.githubusercontent.com/metallb/metallb/v0.13.7/config/manifests/metallb-native.yaml
```

```shell
kubectl wait --namespace metallb-system \
    --for=condition=ready pod \
    --selector=app=metallb \
    --timeout=90s
```

```yaml
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: example
  namespace: metallb-system
spec:
  addresses:
  - 172.19.255.200-172.19.255.250
---
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: empty
  namespace: metallb-system
```

```shell
kubectl apply -f kind-tests/metallb-pool.yaml
```

```shell
istioctl install --set profile=demo -y
```

```shell
kubectl label namespace default istio-injection=enabled
```

```shell
kubectl apply -f kind-tests/idec.yaml
```

```shell
kubectl port-forward services/idec 8080:80
```

```shell
http :8080/export
```

```shell
export RUST_LOG=INFO
```

```shell
cargo run
```

## Run As SystemD Service

```sh
sudo cp coredns-k8s-sync.service /etc/systemd/system/
```

```shell
sudo cat /etc/systemd/system/coredns-k8s-sync.service
```



```shell
sudo systemctl daemon-reload
```

```shell
sudo systemctl enable coredns-k8s-sync
```

```shell
sudo systemctl start coredns-k8s-sync
```

```shell
sudo systemctl stop coredns-k8s-sync
```

```shell
sudo systemctl status coredns-k8s-sync
```

```shell
sudo journalctl -u coredns-k8s-sync -n50
```

## Permisions To Restart CoreDNS Service

```shell
sudo vim /etc/sudoers.d/coredns
```

```shell
%coredns ALL= NOPASSWD: /bin/systemctl start coredns-k8s-sync
%coredns ALL= NOPASSWD: /bin/systemctl stop coredns-k8s-sync
%coredns ALL= NOPASSWD: /bin/systemctl restart coredns-k8s-sync
```

```shell
sudo visudo -c
```

```shell
coredns ALL=(ALL) NOPASSWD: ALL
```