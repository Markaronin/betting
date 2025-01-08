FROM ubuntu:22.04

RUN <<EOF
    apt-get update
    apt-get -y install git curl unzip build-essential sudo zip uuid
EOF
RUN <<EOF
    curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
    unzip awscliv2.zip
    ./aws/install
    rm -r ./awscliv2.zip ./aws
EOF

ARG REMOTE_USER
ARG REMOTE_UID
ARG REMOTE_GID
RUN <<EOF
    useradd ${REMOTE_USER} --create-home
EOF

ENV HOME /home/${REMOTE_USER}

USER ${REMOTE_USER}

RUN <<EOF
    curl https://sh.rustup.rs -sSf | bash -s -- -y
EOF
ENV PATH $HOME/.cargo/bin:$PATH

RUN <<EOF
    mkdir $HOME/.zig
    curl https://ziglang.org/download/0.13.0/zig-linux-x86_64-0.13.0.tar.xz | tar --extract --xz --file - --directory $HOME/.zig --strip-components 1
EOF
ENV PATH $HOME/.zig:$PATH
RUN <<EOF
    cargo install --locked cargo-lambda
EOF