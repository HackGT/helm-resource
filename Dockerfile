FROM scorpil/rust:1.15

# Download kubectl
ADD https://storage.googleapis.com/kubernetes-release/release/v1.3.4/bin/linux/amd64/kubectl \
    /usr/local/bin/kubectl

# Install kubectl
RUN chmod +x /usr/local/bin/kubectl

# Download Helm
ADD http://storage.googleapis.com/kubernetes-helm/helm-v2.1.3-linux-amd64.tar.gz \
    /tmp/helm.tar.gz

# Install Helm
RUN tar -C /tmp -xvf /tmp/helm.tar.gz && \
    mv /tmp/linux-amd64/helm /bin/ && \
    chmod +x /bin/helm

# Copy over data
COPY . /root/helm-resource

# Build the apps
RUN cd /root/helm-resource && \
    cargo build --release && \
    mkdir -p /opt/resource/ && \
    cp target/release/check /opt/resource/check && \
    cp target/release/in /opt/resource/in && \
    cp target/release/out /opt/resource/out
