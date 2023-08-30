FROM rust:1.72 as build
COPY . .
RUN cargo install --path .

FROM gcr.io/distroless/cc-debian12
COPY --from=build /usr/local/cargo/bin/fcaptcha-server /usr/local/bin/fcaptcha-server
CMD ["fcaptcha-server"]
EXPOSE 8080
