FROM quay.io/fedora/fedora:39
RUN dnf install -y beets beets-plugins
COPY target/release/sonar /usr/bin/sonar
ENTRYPOINT ["/usr/bin/sonar"]
