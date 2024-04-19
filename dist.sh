#!/bin/sh

if [ "$VERSION" = "" ]; then
	echo "Missing VERSION"
	exit 1
fi

mkdir -p dist
cargo build --release --target-dir target/ || exit 1
cp target/release/opensubsonic dist/
rpm-assembler \
	--name opensubsonic \
	--summary "opensubsonic cli client" \
	--version $VERSION \
	--arch x86_64 \
	dist/opensubsonic:/usr/bin/opensubsonic:0755 || exit 1
cp *.rpm dist/
