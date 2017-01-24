FROM alpine

# Download Helm
ADD http://storage.googleapis.com/kubernetes-helm/helm-v2.1.3-linux-amd64.tar.gz \
    /tmp/helm.tar.gz

# Install Helm
RUN tar -C /tmp -xvf /tmp/helm.tar.gz && \
    mv /tmp/linux-amd64/helm /bin/ && \
    chmod +x /bin/helm

# Copy over data
COPY ./bin/target/x86_64-unknown-linux-musl/release/helm-resource /opt/resource/
COPY ./scripts/* /opt/resource/
COPY ./ca-certs.crt /etc/ssl/certs/ca-certificates.crt
