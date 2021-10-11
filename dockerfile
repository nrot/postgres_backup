FROM centos:7

WORKDIR /usr/src/app

RUN yum -y install gcc

ADD . .
RUN /bin/sh ./rustup.sh -y
RUN /root/.cargo/bin/cargo build --release
RUN cp target/release/postgres_backup_wal ./