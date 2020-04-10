#!/bin/bash

gnome-terminal -- tail -f log.txt &
cargo build && gnome-terminal --geometry=132x43 -- bash -c "cargo run -- $* 2> log.txt"
