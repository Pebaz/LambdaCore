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

### Grammar

```javascript
// Programs are constructed from one or more functions
Program = { SOI ~ Function+ ~ EOI }

// Functions are identifiers possibly followed by more identifiers or values
Function = { "(" ~ Identifier ~ (Identifier | Value)* ~ ")" }

Identifier = { (ASCII_ALPHANUMERIC | "-" | "_")+ }

// Don't keep the parsed `Value` object
Value = _{ String | Number | Boolean | Null }

Boolean = { "True" | "False" }

Null = { "Null" }

String = ${ "\"" ~ StringContents ~ "\"" }

// Keep the whitespace (literal) characters in the string
StringContents = @{ Character* }

Character = {
    !("\"" | "\\") ~ ANY
    | "\\" ~ ("\"" | "\\" | "/" | "b" | "f" | "n" | "r" | "t")
    | "\\" ~ ("u" ~ ASCII_HEX_DIGIT{4})
}

Number = @{
    "-"?
    ~ ("0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT*)
    ~ ("." ~ ASCII_DIGIT*)?
    ~ (^"e" ~ ("+" | "-")? ~ ASCII_DIGIT+)?
}
```

