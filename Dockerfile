FROM rust:1.57.0

ENV NODE_VERSION=16.13.0
RUN apt install -y curl
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"
RUN node --version
RUN npm --version

WORKDIR /usr/src/zfx-tezos-client
COPY . .

ENV RPC_ADDRESS=""
ENV CONTRACT_ADDRESS=""

RUN npm i --save @taquito/taquito
RUN npm i --save @taquito/signer

RUN cargo build

ENTRYPOINT cargo run -- -r $RPC_ADDRESS -c $CONTRACT_ADDRESS