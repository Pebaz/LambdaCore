<img src="dev/logo/LambdaCore - Logo.png" width=256 />

# LambdaCore
A small Lisp written in Rust.

## Hello World

```clojure
(print "Hello World")
(set 'name "Pebaz")
(prin "Hello ")
(print name)
```

### Building

```bash
git clone https://github.com/Pebaz/LambdaCore
cd LambdaCore
cargo build --release
```

### Running

```bash
# Launch the REPL
./lambda_core

# Run a source file
./lambda_core -f some-file.lcore

# Run a code snippet
./lambda_core -c '(print "Hello World")'
```

### Docs

* Examples: [https://github.com/Pebaz/LambdaCore/tree/master/examples](https://github.com/Pebaz/LambdaCore/tree/master/examples)
* Tutorials: [https://sites.google.com/view/lcore](https://sites.google.com/view/lcore)
* Documentation: [https://github.com/Pebaz/LambdaCore/wiki](https://github.com/Pebaz/LambdaCore/wiki)
* Downloads: [https://github.com/Pebaz/LambdaCore/releases](https://github.com/Pebaz/LambdaCore/releases)
