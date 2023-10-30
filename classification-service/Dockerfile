# Build
FROM docker.io/rust:latest as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

# Deploy
FROM docker.io/debian:stable-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/myapp/target/release/classification-service /usr/local/bin/classification-service
EXPOSE 8080
CMD ["classification-service"]