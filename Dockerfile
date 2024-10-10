FROM --platform=linux/amd64 alpine:3.14 AS base

COPY $HOME/work/wordhooks-rs/wordhooks-rs/target/release/wordhooks-rs ./

CMD ["./wordhooks-rs"]