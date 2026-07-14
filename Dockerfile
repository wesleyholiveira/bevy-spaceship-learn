FROM rust:1.95-bookworm AS builder

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        build-essential \
        libudev-dev \
        libwayland-dev \
        libx11-dev \
        libxcursor-dev \
        libxi-dev \
        libxinerama-dev \
        libxkbcommon-dev \
        libxrandr-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --package spaceship

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        libgl1 \
        libudev1 \
        libvulkan1 \
        libwayland-client0 \
        libwayland-cursor0 \
        libwayland-egl1 \
        libx11-6 \
        libxcursor1 \
        libxi6 \
        libxinerama1 \
        libxkbcommon-x11-0 \
        libxkbcommon0 \
        libxrandr2 \
        mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/spaceship /usr/local/bin/spaceship

ENV RUST_LOG=info

ENTRYPOINT ["spaceship"]
