flasher
=======
JLinkExe wrapper for embedded projects in Rust

Converts elf to binary, calls JLinkExe and flashes MCU in an instant.
Works with `cargo run` and `cargo run --example`.

Installation
------------
* Build and install to `~/.cargo/bin`:

`cargo install --git https://github.com/romixlab/flasher`

Make sure that `arm-none-eabi-objcopy` and `JLinkExe` is in your $PATH.

* Modify `.cargo/config` file:
~~~
[target.thumbv6m-none-eabi]
runner = "jlink-flasher"
~~~

* Create flasher.conf file in the root of your project, each line will be passed to JLinkExe, {bin_path} will be replaced with an actual path. See provided file for reference.

* Run `cargo run` or `cargo run --example <example_name>`, `--release` flag also works.