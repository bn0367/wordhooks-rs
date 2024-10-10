FROM alpine:3.14 AS base

RUN pwd
RUN ls $HOME

COPY $HOME/target/release/wordhooks-rs* ./

CMD ["./wordhooks-rs"]