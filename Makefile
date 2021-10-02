$(VERBOSE).SILENT:

.PHONY: all
all: clean release

.PHONY: clean
clean:
	if [ -d "pkg" ]; then rm -rf pkg; fi
	@echo Done

.PHONY: debug
debug:
	wasm-pack build --dev --target web
	@echo Done

.PHONY: release
release:
	wasm-pack build --release --target web -- --no-default-features
	@echo Done

.PHONY: format
format:
	cargo fmt

