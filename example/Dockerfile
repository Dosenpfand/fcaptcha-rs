FROM rust:1.72 as build
COPY . .
RUN cargo install --example fcaptcha-demo --path .

FROM gcr.io/distroless/cc-debian12
COPY --from=build /usr/local/cargo/bin/fcaptcha-demo /usr/local/bin/fcaptcha-demo
CMD ["fcaptcha-demo"]
EXPOSE 8080
