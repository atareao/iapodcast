###############################################################################
## Backend builder
###############################################################################
FROM rust:alpine3.19 AS builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

RUN apk add --update --no-cache \
            musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release && \
    cp /app/target/release/iapodcast /app/iapodcast


###############################################################################
## Final image
###############################################################################
FROM alpine:3.19

ENV USER=app \
    UID=10001

RUN apk add --update --no-cache \
            tzdata~=2024 && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists*

# Create the user

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/${USER}" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    "${USER}" && \
    mkdir -p /app && \
    chown -R app: /app

# Set the work dir
WORKDIR /app
USER app

# Copy our build
COPY --from=builder /app/iapodcast /app/

CMD ["/app/iapodcast"]
