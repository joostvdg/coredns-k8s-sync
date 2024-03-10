# coredns-k8s-sync

Synchronize CoreDNS config file with DNS entries from Kubernetes resources

## TODO

* [ ] Read in a source file containing DNS records from Kubernetes resources
    * [ X ] Collect this source file from IDEC endpoints
      * [ X ] Configure these endpoints with a configuration file
    * [ ] Detect duplications and remove them (and log them)
    * [ ] Sort the records in each section, so that output is consistent
    * [ ] Merge the records into a single file with source files
* [ ] Set source files in the config file
* [ ] Set destination file in the config file
* [ ] Restart CoreDNS when the destination file changes
* [ ] Integration test, using several test source files and then verifying the output


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