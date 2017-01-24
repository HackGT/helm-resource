FROM busybox:musl
COPY ./bin/target/x86_64-unknown-linux-musl/release/helm-resource /opt/resource/
COPY ./scripts/* /opt/resource/
COPY ./ca-certs.crt /etc/ssl/certs/ca-certificates.crt
COPY ./helm /bin/helm
