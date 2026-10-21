APP         := desktools
VERSION     := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
ARCH        := $(shell uname -m)
PREFIX      ?= /usr/local
DESTDIR     ?=
CARGO       ?= cargo
CARGO_FLAGS ?=

BINDIR      := $(DESTDIR)$(PREFIX)/bin
APPDIR      := $(DESTDIR)$(PREFIX)/share/applications
LICENSEDIR  := $(DESTDIR)$(PREFIX)/share/licenses/$(APP)
PKGDIR      := packaging/arch
DISTDIR     := dist
TARBALL     := $(DISTDIR)/$(APP)-$(VERSION).tar.gz
SOURCEFILES := Cargo.toml Cargo.lock src packaging/desktools.desktop packaging/arch/PKGBUILD README.md LICENSE Makefile .gitignore

.PHONY: all build release run check clippy fmt test install uninstall dist arch clean help

all: build

build:
	$(CARGO) build $(CARGO_FLAGS)

release:
	$(CARGO) build --release $(CARGO_FLAGS)

run:
	$(CARGO) run $(CARGO_FLAGS)

check:
	$(CARGO) check --all-targets

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

fmt:
	$(CARGO) fmt

test:
	$(CARGO) test

install: release
	install -Dm755 target/release/$(APP) $(BINDIR)/$(APP)
	install -Dm644 packaging/$(APP).desktop $(APPDIR)/$(APP).desktop
	install -Dm644 LICENSE $(LICENSEDIR)/LICENSE

uninstall:
	rm -f $(BINDIR)/$(APP)
	rm -f $(APPDIR)/$(APP).desktop
	rm -rf $(LICENSEDIR)

$(TARBALL):
	mkdir -p $(DISTDIR)
	tar --transform 's,^,$(APP)-$(VERSION)/,' -czf $(TARBALL) $(SOURCEFILES)

dist: $(TARBALL)

arch: dist
	@command -v makepkg >/dev/null || { echo "makepkg not found: install base-devel"; exit 1; }
	cp $(TARBALL) $(PKGDIR)/
	cd $(PKGDIR) && makepkg --noconfirm --cleanbuild -f
	mv $(PKGDIR)/$(APP)-$(VERSION)-*.pkg.tar.* $(DISTDIR)/
	install -m755 $(PKGDIR)/pkg/$(APP)/usr/bin/$(APP) $(DISTDIR)/$(APP)-$(ARCH)
	rm -rf $(PKGDIR)/src $(PKGDIR)/pkg $(PKGDIR)/$(APP)-$(VERSION).tar.gz
	@echo "Executable and package written to $(DISTDIR)/"

clean:
	$(CARGO) clean
	rm -rf $(DISTDIR) $(PKGDIR)/src $(PKGDIR)/pkg $(PKGDIR)/$(APP)-$(VERSION).tar.gz

help:
	@echo "Targets:"
	@echo "  build      debug build"
	@echo "  release    optimized build"
	@echo "  run        build and run"
	@echo "  check      cargo check"
	@echo "  clippy     run clippy with -D warnings"
	@echo "  fmt        format the code"
	@echo "  test       run tests"
	@echo "  install    install to PREFIX (default /usr/local, supports DESTDIR)"
	@echo "  uninstall  remove installed files"
	@echo "  dist       source tarball in dist/"
	@echo "  arch       build an Arch package and standalone ELF in dist/"
	@echo "  clean      remove build artifacts"
