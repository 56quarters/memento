include/memento.h:
	cargo build

ctest: include/memento.h test.c
	$(CC) -Iinclude -Ltarget/debug -lmemento -o ctest test.c

clean:
	rm -f ctest

run: ctest
	LD_LIBRARY_PATH=target/debug ./ctest

.PHONY: include/memento.h
