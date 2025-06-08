COMPILER=target/debug/chimocc

CC=cc
CFLAGS=-g -O0 -std=c11 -static
ASFLAGS=-masm=intel

# ホストマシンの環境を検出
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

# Mac(arm)の場合のみDockerを使用
ifeq ($(UNAME_S),Darwin)
ifeq ($(UNAME_M),arm64)
    DOCKER_RUN=docker-compose run --rm dev
else
    DOCKER_RUN=
endif
else
    DOCKER_RUN=
endif

# 共通の実行コマンドを定義
RUN_CMD=$(if $(DOCKER_RUN),$(DOCKER_RUN) $(1),$(1))

$(COMPILER): FORCE
	$(call RUN_CMD,cargo build)

tmp.s: tmp.c
	$(call RUN_CMD,$(COMPILER) $<)

tmp: tmp.s
	$(call RUN_CMD,$(CC) $(CFLAGS) $(ASFLAGS) $< -o $@)

test: $(COMPILER)
	$(call RUN_CMD,./test/test.sh)

cargo_test:
	$(call RUN_CMD,cargo test)

test_all: cargo_test test

clean:
	rm -f tmp.s tmp
	$(call RUN_CMD,cargo clean)

fmt: 
	cargo fmt --all
	cargo clippy --fix --allow-dirty

.PHONY: FORCE test clean test cargo_test test_all fmt
