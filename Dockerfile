FROM gcr.io/distroless/cc-debian12
ARG TARGETARCH
COPY bin/s3web-$TARGETARCH /s3web
ENTRYPOINT ["/s3web"]
