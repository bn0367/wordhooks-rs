FROM alpine:3.14 AS base

COPY target/release/wordhooks-rs* ./

CMD ["./wordhooks-rs"]