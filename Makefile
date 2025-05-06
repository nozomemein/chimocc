COMPILER=target/debug/chimocc

CC=cc
CFLAGS=-g -O0 -std=c11

$(COMPILER): FORCE
	cargo build

tmp.s: tmp.c
	$(COMPILER) $<

tmp: tmp.s
	$(CC) $(CFLAGS) $< -o $@

test: $(COMPILER)
	./test/test.sh

clean:
	rm -f tmp.s tmp
	cargo clean

.PHONY: FORCE test clean