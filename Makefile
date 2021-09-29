prefix ?= /usr
sysconfdir ?= /etc
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

.PHONY: all clean distclean install uninstall update

DEBUG ?= 0
ifeq ($(DEBUG),0)
	ARGS += "--release"
	TARGET = release
endif

VENDORED ?= 0
ifeq ($(VENDORED),1)
	ARGS += "--frozen" "--offline"
endif

WM_BIN=pop-cosmic
WM_BIN_PATH=target/$(TARGET)/$(WM_BIN)
WM_SRC=\
	Cargo.toml \
	Cargo.lock \
	$(shell find src -type f -wholename '*src/*.rs')

PANEL_BIN=pop-cosmic-panel
PANEL_BIN_PATH=panel/target/$(TARGET)/$(PANEL_BIN)
PANEL_SRC=\
	panel/Cargo.toml \
	panel/Cargo.lock \
	$(shell find panel/src -type f -wholename '*src/*.rs')

all: $(WM_BIN_PATH) $(PANEL_BIN_PATH)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar.xz

install: all
	install -D -m 0755 "$(WM_BIN_PATH)" "$(DESTDIR)$(bindir)/$(WM_BIN)"
	install -D -m 0755 "$(PANEL_BIN_PATH)" "$(DESTDIR)$(bindir)/$(PANEL_BIN)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(WM_BIN)"
	rm -f "$(DESTDIR)$(bindir)/$(PANEL_BIN)"

update:
	cargo update

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcfJ vendor.tar.xz vendor
	rm -rf vendor

$(WM_BIN_PATH): $(WM_SRC)
ifeq ($(VENDORED),1)
	tar pxf vendor.tar.xz
endif
	cargo rustc $(ARGS) --bin $(WM_BIN) -- \
	    -C link-arg=-Wl,-rpath,/usr/lib/x86_64-linux-gnu/mutter-8

$(PANEL_BIN_PATH): $(PANEL_SRC)
ifeq ($(VENDORED),1)
	tar pxf vendor.tar.xz
endif
	cargo build $(ARGS) --bin $(PANEL_BIN) --manifest-path panel/Cargo.toml
