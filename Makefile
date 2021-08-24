prefix ?= /usr
sysconfdir ?= /etc
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

SRC = Cargo.toml Cargo.lock Makefile $(shell find src -type f -wholename '*src/*.rs')

.PHONY: all clean distclean install uninstall update

BIN=pop-cosmic

DEBUG ?= 0
ifeq ($(DEBUG),0)
	ARGS += "--release"
	TARGET = release
endif

VENDORED ?= 0
ifeq ($(VENDORED),1)
	ARGS += "--frozen" "--offline"
endif

all: target/$(TARGET)/$(BIN)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar.xz

install: all
	install -D -m 0755 "target/$(TARGET)/$(BIN)" "$(DESTDIR)$(bindir)/$(BIN)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN)"

update:
	cargo update

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcfJ vendor.tar.xz vendor
	rm -rf vendor

target/c/wrapper.o: src/wrapper.c
	mkdir -p target/c
	$(CC) -c $^ -o $@ -Werror \
		-Wl,-rpath /usr/lib/x86_64-linux-gnu/mutter-8 \
		$(shell pkg-config --cflags --libs libmutter-8)

target/c/libwrapper.a: target/c/wrapper.o
	ar -rc $@ $^

target/$(TARGET)/$(BIN): $(SRC) target/c/libwrapper.a
ifeq ($(VENDORED),1)
	tar pxf vendor.tar.xz
endif
	cargo build $(ARGS)
