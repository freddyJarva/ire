# ire
Interactive regex built in Rust

Test your regex pattern on a file, and get instant updates on what lines match, and what parts of the line are captured by groups.
There also exists an option to output the captured groups in csv format.

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

