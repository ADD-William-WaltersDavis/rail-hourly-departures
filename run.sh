#!/bin/bash

set -e

mkdir -p data

time cargo run --release -- \
	--input-file-path=input/timetables_2025_Q4_Rail.cif \
    --operating-day=tuesday \
	--output-directory=data