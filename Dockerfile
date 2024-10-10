FROM alpine:3.14 AS base

RUN ls $HOME
RUN ls

COPY $HOME/target/release/wordhooks-rs* ./

CMD ["./wordhooks-rs"]