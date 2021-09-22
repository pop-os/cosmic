# Basic Makefile

# Retrieve the UUID from ``metadata.json``
UUID = $(shell grep -E '^[ ]*"uuid":' ./metadata.json | sed 's@^[ ]*"uuid":[ ]*"\(.\+\)",[ ]*@\1@')

ifeq ($(XDG_DATA_HOME),)
XDG_DATA_HOME = $(HOME)/.local/share
endif

ifeq ($(strip $(DESTDIR)),)
INSTALLBASE = $(XDG_DATA_HOME)/gnome-shell/extensions
else
INSTALLBASE = $(DESTDIR)/usr/share/gnome-shell/extensions
endif
INSTALLNAME = $(UUID)

SRC = applications.js \
      extension.js \
      metadata.json \
      overview.js \
      prefs.js \
      schemas/gschemas.compiled \
      schemas/org.gnome.shell.extensions.pop-cosmic.gschema.xml \
      settings.js \
      stylesheet.css \
      topBarButton.js

$(info UUID is "$(UUID)")

.PHONY: all clean install zip-file

all: $(SRC)
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
