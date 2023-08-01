#!/usr/bin/env sh

mkdir .cargo
cargo vendor > .cargo/config

version=$1
tar czf distrobox-boost-$version.tar.gz --transform "s,^,distrobox-boost-$version/," --exclude mk_cargo_vender.sh --exclude distrobox-boost-*.tar.gz --exclude .git --exclude target * .cargo
rm -rf vendor/
rm -rf .cargo/
