FROM --platform=linux/amd64 alpine:3.14 AS base

RUN echo $HOME
RUN ls /home/runner/work/wordhooks-rs/wordhooks-rs

COPY $HOME/target/release/wordhooks-rs* ./

CMD ["./wordhooks-rs"]