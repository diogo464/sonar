FROM quay.io/fedora/fedora:39
RUN dnf install -y beets beets-plugins ffmpeg-free && dnf clean all
COPY target/release/sonar /usr/bin/sonar
ENTRYPOINT ["/usr/bin/sonar"]
