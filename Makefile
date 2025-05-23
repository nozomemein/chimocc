COMPILER=target/debug/chimocc

CC=cc
CFLAGS=-g -O0 -std=c11 -static

$(COMPILER): FORCE
	cargo build

tmp.s: tmp.c
	$(COMPILER) $<

tmp: tmp.s
	$(CC) $(CFLAGS) $< -o $@

test: $(COMPILER)
	./test/test.sh

cargo_test:
	cargo test

test_all: cargo_test test


clean:
	rm -f tmp.s tmp
	cargo clean

fmt: 
	cargo fmt --all
	cargo clippy --fix --allow-dirty

.PHONY: FORCE test clean test cargo_test test_all fmt