include/memento.h:
	cargo build

c-tests: include/memento.h c-tests/main.c
	$(CC) -Wall -pedantic -Iinclude -Ltarget/debug -lmemento -o target/c-tests c-tests/main.c

clean:
	rm -f target/c-tests

run: c-tests
	LD_LIBRARY_PATH=target/debug ./target/c-tests

.PHONY: include/memento.h
