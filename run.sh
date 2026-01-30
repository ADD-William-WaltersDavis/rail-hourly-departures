#!/bin/bash

set -e

mkdir -p data

time cargo run --release -- \
	--input-file-dir=input \
    --operating-day=tuesday \
	--operating-week=260112 \
	--output-directory=data