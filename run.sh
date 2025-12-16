#!/bin/bash

set -e

mkdir -p data

time cargo run --release -- \
	--input-dir-path=input \
    --operating-day=tuesday \
	--output-directory=data