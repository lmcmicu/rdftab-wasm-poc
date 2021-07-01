MAKEFLAGS += --warn-undefined-variables
SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := webapp
.DELETE_ON_ERROR:
.SUFFIXES:

release_dir = target/wasm32-unknown-unknown/release

.PHONY: webapp
webapp: www/rdftab_wasm_poc.js www/rdftab_wasm_poc_bg.wasm

www/rdftab_wasm_poc.js: $(release_dir)/rdftab_wasm_poc.js
	rm -f $@
	cp -f $< www/

www/rdftab_wasm_poc_bg.wasm: $(release_dir)/rdftab_wasm_poc_bg.wasm
	rm -f $@
	cp -f $< www/

$(release_dir)/rdftab_wasm_poc.js: $(release_dir)/rdftab_wasm_poc_bg.wasm

$(release_dir)/rdftab_wasm_poc_bg.wasm: $(release_dir)/rdftab_wasm_poc.wasm
	wasm-bindgen --target web --no-typescript --out-dir $(release_dir) $<
	wasm-gc $@

$(release_dir)/rdftab_wasm_poc.wasm: $(wildcard src/*.rs)
	cargo build --target wasm32-unknown-unknown --release
	wasm-gc $@

clean:
	cargo clean
	rm -f www/rdftab_wasm_poc.js www/rdftab_wasm_poc_bg.wasm
