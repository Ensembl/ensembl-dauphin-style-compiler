# !IMPORTANT! Run this Dockerfile from the parent directory to include all dependent packages
# To do so, run the following command from the parent directory: `docker build -f peregrine-ensembl/Dockerfile .`


#### FIRST BUILDER CONTAINER: BUILD WASM FROM RUST ####
FROM rust:1.61 AS rust-builder

COPY . /peregrine-build

WORKDIR /peregrine-build/peregrine-ensembl

# build artifacts will be saved to /peregrine-build/peregrine-ensembl/pkg
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh && \
    cargo build && \
    RUSTFLAGS="--cfg=console" wasm-pack build --target web --release

#### SECOND BUILDER CONTAINER: BUILD THE JAVASCRIPT CODE ####
FROM node:16.13.0 AS node-builder

RUN mkdir -p /srv/ensembl-client-integration
WORKDIR /srv/ensembl-client-integration/

# Clone ensembl-genome-browser repo and ensembl-client repo
RUN git clone https://github.com/Ensembl/ensembl-client.git && \
    git clone https://github.com/Ensembl/ensembl-genome-browser.git



# Step 1: Build ensembl-genome-browser

WORKDIR /srv/ensembl-client-integration/ensembl-genome-browser
COPY --from=rust-builder /peregrine-build/peregrine-ensembl/pkg/* ./src/peregrine/

RUN npm ci --loglevel warn && \
    npm run build


# Step 2: Build ensembl-client using the build artifacts from ensembl-genome-browser

WORKDIR /srv/ensembl-client-integration/ensembl-client

# The command below, among other things, updates ensembl-client's package.json
# to use the package just built by ensembl-genome-browser as a dependency.
# As a consequence, "npm install" must be used instead of "npm ci", because package.json is now the source of truth
RUN npm install -g json && \
    json -I -f package.json -e "this.dependencies['ensembl-genome-browser']='file:/../ensembl-genome-browser'" && \
    npm install && \
    npm run build


#### IMAGE FOR RUNNING THE CODE ####
FROM node:16.13.0-alpine AS runner

WORKDIR /srv/ensembl-client-integration/

ENV NODE_ENV=production
COPY --from=node-builder /srv/ensembl-client-integration/ensembl-client/package* ./
COPY --from=node-builder /srv/ensembl-client-integration/ensembl-client/dist ./dist

RUN npm ci --only=production --ignore-scripts

EXPOSE 8080
CMD [ "node", "dist/server/server.js" ]
