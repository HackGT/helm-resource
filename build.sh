#!/usr/bin/env bash
set -euo pipefail
set -x

# copy over the ca cert
cp /etc/ssl/certs/ca-certificates.crt ca-certs.crt

# get the helm binary
curl 'http://storage.googleapis.com/kubernetes-helm/helm-v2.1.3-linux-amd64.tar.gz' \
     -o /tmp/helm.tar.gz
tar -C /tmp -xvf /tmp/helm.tar.gz
mv /tmp/linux-amd64/helm .
chmod +x helm

# build shit
pushd bin
cargo build --release
popd
