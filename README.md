# Bunnylol

My interpretation of the bunnylol tool used at Meta and other tech companies.

## Features
* Saves sites to YAML config file
* Simple web UI to add and remove sites
* Accepts placeholder ("{}") in URLs. It will be replaced by your query minus the first word.

## Usage

`cargo build && cargo run` should install deps and run the development server.
You can use `cargo build -r` to build a "release", which is optimized.
Configs can be overriden by environment variables:
    `ROCKET_CONFIG_FILE=~/.bunnylol.yaml ./target/release/bunnylol`

## Disclaimer
This is backed by a YAML file, any real usage other than personal would probably cause issues and wouldn't be very fast at all.
It should probably use a DB, but this is enough for my usecase for now.
