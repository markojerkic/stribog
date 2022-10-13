# STRIBOG
A simple file tree traverser with simple forbidden list filter

To build, simply run:
```
cargo build -r
```

```
Usage: stribog [OPTIONS] --root <ROOT>...

Options:
  -r, --root <ROOT>...         List of root directories to search
  -f, --forbidden <FORBIDDEN>  List of forbiden directory names. Any dir which name starts with any of the entries will be skipped and not walked into
  -m, --max-depth <MAX_DEPTH>  Max depth to walk [default: 2147483647]
  -u, --use-cache              Opt in to use cached valus. Always a little bit behind, but should be faster
  -i, --is-linux               Opt in to use cached valus. Always a little bit behind, but should be faster
  -h, --help                   Print help information
  -V, --version                Print version information
```
