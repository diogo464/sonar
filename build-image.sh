#!/usr/bin/sh

mkdir -p target
cargo build --release --target-dir target || exit 1
docker build -t git.d464.sh/code/sonar:latest -f Containerfile .

if [ "$PUSH" = "1" ]; then
	docker push git.d464.sh/code/sonar:latest
fi
