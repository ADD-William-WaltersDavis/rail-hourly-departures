#!/bin/bash

set -e

mkdir -p data

time cargo run --release -- \
	--input-file-dir=input \
    --operating-day=tuesday \
	--output-directory=data