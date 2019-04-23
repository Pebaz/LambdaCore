<img src="dev/logo/LambdaCoreLogo.png" width=256 />

# LambdaCore
A small Lisp written in Rust.

### Plan

- [x] Read Lisp file in. File extension: `.lcore`
- [x] Parse Lisp syntax.
- [x] Agree upon data types.
- [x] Call function from Rust in LambdaCore.
- [ ] Print formatted syntax to HTML with colors.
- [ ] Infinitely Extend. 😃
- [ ] Symbol Table (A hidden variable within the scope)
- [x] Built-in functions

##### Quoting

This is going to be an interesting undertaking. Look at quoting, quasi-quoting, back-quoting?

### Runtime

 * Plant variable into root namespace named `SYMBOL_TABLE`.
	* Use this variable to manage the language from within itself.
	* Scopes are lists of symbols just like Craft (aka Wing v1).
 * [Tail-call recursion optimization](https://github.com/murarth/ketos/blob/master/docs/README.md)

### Data Types

 * Null
 * Identifier
 * Boolean
 * Int (x64)
 * Float (x64)
 * String
 * Array
 * Func
 * Struct

### Syntax

```Lisp
(print "Hello World")

(loop 10 i [
	(print (format "Iteration: {i}"))
	(set name "Pebaz")
	(print f"Hello {name}")])

(print (* 10 2))

(defn get-age [person]
	(ret person/age))
```
