#!/usr/bin/sh

mkdir -p src/lib/server/pb
protoc \
	--plugin=./node_modules/.bin/protoc-gen-ts_proto \
	--ts_proto_out=./src/lib/server/pb \
	--ts_proto_opt=esModuleInterop=true \
	--ts_proto_opt=outputServices=nice-grpc,outputServices=generic-definitions,useExactTypes=false \
	--proto_path=../sonar-grpc/ sonar.proto
