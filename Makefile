FLAGS = RUSTFLAGS='-C target-cpu=native'
FEATURES = --features "benchmark"
BENCHING = sse41 sse42 avx2

.PHONY: test
test:
	$(FLAGS) cargo test $(FEATURES)

.PHONY: bench
bench:
	$(FLAGS) cargo bench $(FEATURES)

.PHONY: bench-release $(BENCHING)
bench-release: $(BENCHING)

$(BENCHING):
	$(FLAGS) cargo run $(FEATURES) --release --bin bench $@
