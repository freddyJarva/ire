# ire

Interactive regex built in Rust

Test your regex pattern on a file, and get instant updates on what lines match, and what parts of the line are captured by groups.
There also exists an option to output the captured groups in csv format.

Only works for linux distributions at the moment.

```
interactive_regex 1.0

USAGE:
    ire [OPTIONS] <FILENAME>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -g, --glob <GLOB>        use glob pattern to read from multiple files
    -o, --output <OUTPUT>    write result to file

ARGS:
    <FILENAME>
```


## Install
Right now, the only way to install is to build from source.
You need rust. check out this page for east install instructions: https://www.rust-lang.org/tools/install
Efter that, just clone this repo and build:

```sh
git clone https://github.com/freddyJarva/ire.git
cd ./ire
cargo build --release
```

the built executable can then be found at: `./target/release/ire`
