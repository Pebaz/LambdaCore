# LambdaCore
A small Lisp written in Rust.

### Plan

1. Read Lisp file in. File extension: `.lc`
2. Parse Lisp syntax.
3. Agree upon data types.
4. Call function from Rust in LambdaCore.
5. Print formatted syntax to HTML with colors.
6. Infinitely Extend. 😃

### Runtime

 * Plant variable into root namespace named `SYMBOL_TABLE`.
	* Use this variable to manage the language from within itself.
	* Scopes are lists of symbols just like Craft (aka Wing v1).

### Data Types

 * x
 * y
 * z

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