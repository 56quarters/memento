build: include/memento.h

include/memento.h:
	RUSTUP_TOOLCHAIN=nightly cbindgen --config cbindgen.toml --output $@

rust:
	RUSTUP_TOOLCHAIN=nightly cargo build

ctest: rust include/memento.h c-tests/main.c
	$(CC) -Wall -fsanitize=undefined -Fsanitize=address -pedantic -Iinclude \
		-Ltarget/debug -lmemento -o target/$@ c-tests/main.c
	LD_LIBRARY_PATH=target/debug ./target/ctest

setup:
	RUSTUP_TOOLCHAIN=nightly cargo install cbindgen

clean:
	rm -f target/ctest include/memento.h

.PHONY: include/memento.h rust
