FROM centos:7

WORKDIR /usr/src/app

RUN yum -y install gcc

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > ./rustup.sh
RUN /bin/sh ./rustup.sh -y

ADD . .

RUN /root/.cargo/bin/cargo build --release
RUN cp target/release/postgres_backup_wal ./