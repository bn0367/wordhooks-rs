FROM --platform=linux/amd64 alpine:3.14 AS base

RUN pwd
RUN ls /
RUN ls
COPY target/release/wordhooks-rs ./

CMD ["./wordhooks-rs"]