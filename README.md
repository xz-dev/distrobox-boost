# distrobox_boost

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)

A container image builder tool for Open Container Initiative (distrobox/toolbox, also podman/docker).  

Distrobox is good enough in running softwore, but compare to Package Manager such as APT, pacman, etc, it miss a faster, agile and Cloud Native way to use Open Container Initiative (OCI).

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
  - [Replace the builder of distrobox-assemble](#replace-the-builder-of-distrobox-assemble)
  - [Build separate file for distrobox-assemble](#build-separate-file-for-distrobox-assemble)
  - [Create distrobox image by command](#create-distrobox-image-by-command)  
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [Donation](#donation)
- [License](#license)

## Background

When I tried to use distrobox, it's not because of lacking packages, most users can package an app if they want, it's not that hard. I used distrobox as my build environment, such as building mono in RISC-V.

> But as you know, the init for distrobox is too slow. I wasted a lot of time waiting for distrobox to tell me everything is ready. Sometimes distrobox init fails when the network is bad or wrong package name, etc.

So I decided to code this tool to save my time and help more users use distrobox and share their packages. Because I know many packages are only made for specific distros, and some distros like Gentoo, Void Linux, don't have many devs to package apps. I want to use them but also need some must-have apps.   

Why I choice Rust but not bash:
1. Easy write tests
2. No runtime dependence issues
3. faster!

The goals of this tool are:

1. Build images the right way for distrobox.
2. Run distrobox easily.  
3. Manage distrobox containers easily.
4. Manage and share containers easily and properly.  
5. Update packages in containers quickly and easily.
6. You tell me.

## Install

### From source

```sh
$ git clone git@github.com:xz-dev/distrobox-boost.git
$ cd distrobox-boost
$ cargo build --release
$ ls target/release/distrobox-boost
```

### From OBS
[home:xz:distrobox-boost](https://build.opensuse.org/package/show/home:xz:distrobox-boost/distrobox-boost)

### From GitHub Release

TODO

### From Docker Hub 

TODO

## Usage

### Replace the builder of distrobox-assemble

1. You need an ini file for distrobox-assemble like [tests/files/example.ini](https://github.com/xz-dev/distrobox-boost/blob/main/tests/files/example.ini) (more info in [distrobox README](https://github.com/89luca89/distrobox/blob/main/docs/usage/distrobox-assemble.md))
   
2. Run command
   ```sh
   $ target/release/distrobox-boost --input ./tests/files/example.ini --output ./tests/files/example_new.ini
   ```
3. distrobox-assemble
   ```sh
   $ distrobox-assemble --file tests/files/example_new.ini create
   $ distrobox list
   ```
   
### Build separate file for distrobox-assemble

1. You need an ini file for distrobox-assemble like [tests/files/example.ini](https://github.com/xz-dev/distrobox-boost/blob/main/tests/files/example.ini) (more info in [distrobox README](https://github.com/89luca89/distrobox/blob/main/docs/usage/distrobox-assemble.md))
  
2. Run command
   ```sh
   $ target/release/distrobox-boost --input ./tests/files/example.ini --output-dir ./tests/files/example_out/
   ```
3. distrobox-assemble
   ```sh
   $ distrobox-assemble --file tests/files/example/arch.ini create
   $ distrobox list
   ```

### Create distrobox image by command

+ Run the package like nix-shell
  ```sh
  $ distrobox-boost fish -c 'ls -la'
  ```
+ Run command in the package's container like nix-env
  ```sh
  $ distrobox-boost fish --run bash -c "ls -la"
  ```

### Pin/Unpin image to avoid clean

podman system prune -a and clean all your container data?

```sh
$ target/release/distrobox-boost --input ./tests/files/example.ini --pin
$ target/release/distrobox-boost --input ./tests/files/example.ini --unpin
$ target/release/distrobox-boost --input ./tests/files/example.ini --output ./tests/files/example_new.ini --pin # It also can use with other args
```

## Roadmap

- [ ] Build image from Dockerfile
- [x] Create distrobox image by command args  
- [ ] Update packages in image
- [ ] Record packages in container
- [ ] Backup/Restore from disk for sharing your container
- [ ] Full tests

## Contributing

Feel free to dive in! Open issues or PRs. Before you code, please discuss in issues first, because I may already be working on something without you knowing. I don't want to waste your time.

## Donation

- [Ko-fi](https://ko-fi.com/xzdev/goal?g=0)
- [爱发电](https://afdian.net/a/inkflaw) 

## License

[MIT](LICENSE)
