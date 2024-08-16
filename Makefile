TARGET = aarch64-unknown-linux-gnu
PACKAGE = quarky
DEBIAN_VERSION = 0.0.2
DEBIAN_REVISION = 1
DEBIAN_ARCH = arm64
DEB_NAME = ${PACKAGE}_${DEBIAN_VERSION}-${DEBIAN_REVISION}_${DEBIAN_ARCH}.deb
HOST = dagger.local

.PHONY: deploy

deploy:
	debian-sysroot-build --target ${TARGET} --package ${PACKAGE} --features pkg-config --install-package libc6 --install-package libc6-dev --install-package linux-libc-dev --install-package libgcc-12-dev --install-package libopus-dev
	cargo deb --target ${TARGET} --no-build --no-strip
	deploy-deb target/${TARGET}/debian/${DEB_NAME} ${HOST}