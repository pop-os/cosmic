# Basic Makefile

# Retrieve the UUID from ``metadata.json``
UUID = $(shell grep -E '^[ ]*"uuid":' ./metadata.json | sed 's@^[ ]*"uuid":[ ]*"\(.\+\)",[ ]*@\1@')

ifeq ($(strip $(DESTDIR)),)
INSTALLBASE = $(HOME)/.local/share/gnome-shell/extensions
else
INSTALLBASE = $(DESTDIR)/usr/share/gnome-shell/extensions
endif
INSTALLNAME = $(UUID)

$(info UUID is "$(UUID)")

.PHONY: all clean install zip-file

all: extension.js metadata.json prefs.js schemas/org.gnome.shell.extensions.pop-cosmic.gschema.xml schemas/gschemas.compiled
	rm -rf build
	for i in $^ ; do \
		mkdir -p build/$$(dirname $$i) ; \
		cp $$i build/$$i ; \
	done

schemas/gschemas.compiled: schemas/*.gschema.xml
	glib-compile-schemas schemas

clean:
	rm -rf build

install: all
	rm -rf $(INSTALLBASE)/$(INSTALLNAME)
	mkdir -p $(INSTALLBASE)/$(INSTALLNAME)
	cp -r build/* $(INSTALLBASE)/$(INSTALLNAME)/

zip-file: all
	cd build && zip -qr "../$(UUID)$(VSTRING).zip" .
