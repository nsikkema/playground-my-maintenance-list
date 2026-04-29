# Build Web Data
FROM node:25.9.0-trixie AS web-builder
WORKDIR /build
COPY . .
RUN cd crates/lib-web && npm install
RUN cd crates/lib-web && npm run build

# Build Rust Project
FROM rust:1.95.0-trixie AS rust-builder
WORKDIR /build
COPY . .
COPY --from=web-builder /build/crates/lib-web/dist /build/crates/lib-web/dist
ENV SKIP_NPM_BUILD=true
ENV SKIP_RUSTFMT=true
RUN cargo build --release

# Build Rust License List
FROM rust:1.95.0-trixie AS rust-license
WORKDIR /build
RUN cargo install cargo-about --version 0.8.4 --locked
COPY . .
RUN cargo about generate -c .cargo-about/about.toml -o RUST_LICENSE.txt .cargo-about/about.hbs

# Combine Projects
# Use distroless image: no shell, no package manager, minimal attack surface
FROM gcr.io/distroless/cc-debian13 AS runtime
WORKDIR /app
COPY --from=web-builder /build/crates/lib-web/dist/LICENSE.txt WEB_LICENSE.txt
COPY --from=rust-builder /build/target/release/server server
COPY --from=rust-license /build/RUST_LICENSE.txt RUST_LICENSE.txt
# Run as non-root user (distroless provides uid 65532 "nonroot")
USER nonroot:nonroot
EXPOSE 3000
ENTRYPOINT ["./server"]
