FROM gcr.io/distroless/base-debian11
ARG TARGETARCH
COPY bin/s3web-$TARGETARCH /s3web
ENTRYPOINT ["/s3web"]
