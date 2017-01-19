FROM busybox:musl

# Download Helm
ADD http://storage.googleapis.com/kubernetes-helm/helm-v2.1.3-linux-amd64.tar.gz \
    /tmp/helm.tar.gz

# Install Helm
RUN tar -C /tmp -xvf /tmp/helm.tar.gz && \
    mv /tmp/linux-amd64/helm /bin/ && \
    chmod +x /bin/helm

# Copy over data
COPY ./build-output/x86_64-unknown-linux-musl/release/check /opt/resource/check
COPY ./build-output/x86_64-unknown-linux-musl/release/in /opt/resource/in
COPY ./build-output/x86_64-unknown-linux-musl/release/out /opt/resource/out
