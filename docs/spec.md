# Speak language specification, v0.2

Status: draft for ticket SPK-101. Version 0.2 is the target of milestone M1 in the [roadmap](roadmap.md).

This document says what a Speak program means. It does three things:

- It specifies v0.2 as the roadmap decided it.
- Where today's interpreter differs, a **Current implementation** note says how, and names the ticket that closes the gap. A gap with no ticket says so, and is listed in [Open questions](#13-open-questions).
- Features that exist only as roadmap tickets (async/await, modules) get a short **Planned** section and no syntax of their own.

Every example marked "runs today" was run with the current interpreter (`speak run file.spk`, with `SPEAK=sw` for Swahili) and the output shown is what it printed. Examples marked "v0.2 only" do not run today.

## 1. Programs and locales

A Speak program is a UTF-8 text file made of expressions. Running it evaluates the top-level expressions in order. Everything in Speak is an expression and has a value, though some forms (function definitions, `for`, assignments, calls to `println`) are used mainly for their effect.

Every word that a learner can read in source code comes from a locale file, not from the interpreter: keywords, type names, builtin names and error messages. This document refers to such words by their locale key, for example `literals.if`, and shows the English and Swahili words in [section 2.3](#23-words-from-the-locale). A new language adds its own words by editing a YAML file. No English word is special.

Punctuation and operators (`? ! -> : . .. ( ) [ ] { } + - * / % & | < > = ~ ,`) are the same in every locale.

> **Current implementation.** One locale is active for the whole process, chosen by the `SPEAK` environment variable (`en` by default, `sw` for Swahili). A file cannot say which language it is written in. SPK-801 replaces this with a per-file locale, and SPK-807 lets one file declare several locales (see [2.4](#24-reserved-words-and-mixed-locales)).

## 2. Lexical structure

### 2.1 Source text and layout

- Source is processed line by line. A token's position is `line:column`, both counted from 1, the column counted in characters.
- Whitespace separates tokens and is otherwise ignored, with one exception: **indentation decides where a block ends** (see [3.3](#33-blocks-and-line-ends)). Use spaces. A tab counts as one column.
- A comment starts at `//` outside a string and runs to the end of the line. Blank lines and comment-only lines are ignored.

### 2.2 Tokens

| Token | Form |
| --- | --- |
| Identifier | An ASCII letter or `_`, then ASCII letters, digits or `_`. |
| Number | Digits, optionally grouped with `_` (`1_000`), optionally followed by `.` and digits (`3.14`). No exponent, no sign. |
| String | Text between double quotes on one line. There are no escape sequences, so a string cannot contain `"`. |
| Empty literal | `()` |
| Empty identifier | `_` followed by a space. Used where a value is thrown away. |
| Type name | The locale words for `types.number`, `types.bool`, `types.string`, and array types `[]T` (for example `[]number`, `[][]string`). |
| Punctuation | `, : :: . .. -> ? !` and the brackets `( ) [ ] { }`. |
| Operators | `+ - * / % & \| < > = ~`, and the locale word `literals.is`. |

A number literal never includes a sign. `-5` is the operator `-` applied to `5`; write a negative number as `~5` or `0 - 5`.

An integer literal has no `.`; a float literal has one. This follows the numbers decision (SPK-108, [5.1](#51-numbers)).

> **Current implementation.** Every number is an `f64`; `7` and `7.0` are the same value. SPK-108 adds `i64` integers.

> **Current implementation.** Identifiers are ASCII only, so `é = 1` or an Amharic name is a syntax error, although strings may hold any Unicode text. Whether v0.2 identifiers may use Unicode letters is open ([Open questions](#13-open-questions), item 1).

> **Current implementation.** The empty string `""` crashes the lexer (an integer underflow panic). This is a bug with no ticket yet.

### 2.3 Words from the locale

| Locale key | English | Swahili | Role |
| --- | --- | --- | --- |
| `literals.true` | `true` | `kweli` | boolean literal |
| `literals.false` | `false` | `ongo` | boolean literal |
| `literals.if` | `if` | `kama` | conditional |
| `literals.for` | `for` | `kwa` | loop |
| `literals.in` | `in` | `katika` | loop |
| `literals.break` | `break` | `vunja` | leave a loop |
| `literals.continue` | `continue` | `endelea` | next loop turn |
| `literals.is` | `is` | `ni` | assignment |
| `types.number` | `number` | `nambari` | type name |
| `types.bool` | `bool` | `bulin` | type name |
| `types.string` | `string` | `mlolongo` | type name |
| `types.object` | `object` | `kifaa` | type name |
| `types.function` | `function` | `kazi` | type name |
| `types.array` | `array` | `safu` | type name (used when printing; array types are written `[]T`) |
| `builtins.print` | `print` | `andika` | builtin |
| `builtins.println` | `println` | `andika_laini` | builtin |
| `builtins.sprint` | `sprint` | `mlolongo_andika` | builtin |
| `builtins.len` | `len` | `urefu` | builtin |
| `builtins.mod` | `mod` | `moduli` | builtin |

The `types.native_function` word (`native function`, `kazi asilia`) names builtins when they are printed. It is not written in source.

### 2.4 Reserved words and mixed locales

In a file with one locale, the keywords (`literals.*`) and type names (`types.number`, `types.bool`, `types.string`) of that locale cannot be used as names. Words from other locales are ordinary identifiers: in an English program `kama` is a valid variable name. Builtin names (`builtins.*`) are ordinary bindings and can be shadowed.

In a file that declares several locales (SPK-807), the keywords and type names of **every** declared locale are reserved, and using one as a name is a lex-time error that names both meanings. The type words are `number`, `bool`, `string`, `object`, `function` and `array` in English, and their Swahili equivalents. Builtin and std names can still be shadowed. The error must explain why a name that worked before became reserved when a locale was added.

> **Current implementation.** Only `types.number`, `types.bool` and `types.string` are recognised as type words. `object`, `function` and `array` are ordinary identifiers. The reserved set therefore grows when SPK-807 lands.

## 3. Grammar

### 3.1 Notation

EBNF: `|` choice, `[ ]` optional, `{ }` zero or more, `( )` grouping, quoted text is literal punctuation. Words in capitals are tokens from [section 2](#2-lexical-structure): `IDENT`, `NUMBER`, `STRING`, and the locale words `IF`, `FOR`, `IN`, `IS`, `TRUE`, `FALSE`, `BREAK`, `CONTINUE`. `TYPENAME` is a type word.

### 3.2 Grammar

```ebnf
program       = { expression } ;

expression    = logical [ IS expression ] ;               (* assignment: lowest precedence *)
logical       = comparison { ( "&" | "|" ) comparison } ;
comparison    = additive { ( "<" | ">" | "=" ) additive } ;
additive      = multiplicative { ( "+" | "-" ) multiplicative } ;
multiplicative= modulo { ( "*" | "/" ) modulo } ;
modulo        = access { "%" access } ;                   (* binds tighter than "*" *)
access        = operand { "." IDENT } ;                   (* field access: highest precedence *)

operand       = "~" operand | application ;
application   = primary { argument } ;                    (* arguments on the same line *)
argument      = operand-no-call { binary-op operand-no-call } ;

primary       = NUMBER | STRING | TRUE | FALSE | "()" | "_"
              | BREAK | CONTINUE
              | IDENT [ index | slice | object-fields ]
              | "(" expression ")"
              | array | if | for | function ;

index         = "[" expression "]" ;
slice         = "[" [ expression ] ".." [ expression ] "]" ;
array         = "[" [ expression { "," expression } [ "," ] ] "]" ;
object-fields = "{" { field } "}" ;
field         = IDENT ( "," | IDENT [ "," ] ) ;           (* "x," or "x y," or "x y" before "}" *)

if            = IF expression [ "?" expression ] [ "!" expression ] ;
for           = FOR IDENT IN expression block ;
function      = IDENT ":" [ params ] "->" type block ;
params        = group { "," group } ;
group         = IDENT { "," IDENT } type ;                (* "a, b number" or "n number" *)
type          = TYPENAME | "[]" type | IDENT ;            (* IDENT names an object type *)
block         = { expression } ;                          (* by indentation, see 3.3 *)
```

Notes on the grammar:

- `operand-no-call` is `operand` with the `application` alternative replaced by `primary`: inside an argument list, a name does not start a nested call. So `f g x` passes `g` and `x` to `f`. Use parentheses to nest: `f (g x)`.
- A call has no brackets. `fib n - 1` calls `fib` with one argument, `n - 1`, because an argument extends over any binary operators that follow it. Write `(fib n - 1) + (fib n - 2)`, as the samples do.
- Binary operators from loosest to tightest: `is`, `&` `|`, `<` `>` `=`, `+` `-`, `*` `/`, `%`, `.`. Operators of one level group left to right. Note that `%` binds tighter than `*`: `2 * 7 % 4` is `2 * (7 % 4)`.
- An `object-fields` block follows an identifier directly: `Person { name, age years, }`. A field `x,` takes its value from the variable `x`. A field `x y,` takes it from the variable `y`. The right-hand side is always a variable name, never a literal or expression.
- On the left of `is` the only legal forms are an identifier, `_`, an indexed element `a[i]`, or a field `object.field`. Anything else is a runtime error.
- In `if`, the arms may appear in either order and each may be left out. They may continue on the next line.
- `function` is a definition: `name: params -> type`, then the body. The same `IDENT ":"` starts every definition, so a name followed by a colon is always a function.
- `_` is the empty identifier. `_ is expr` evaluates `expr` and discards the result.

> **Current implementation.**
> - `&` and `|` are not part of the precedence climb: they work between two operands (`a | b`, `(x < 1) & (y < 2)`) but not in a chain or next to comparison operators (`x < 1 & y < 2` is a syntax error). There is no ticket yet; the grammar above is the intended v0.2 form.
> - `~` can be applied only to a literal, and only where an operand starts. `~5` and `(~5)` work; `~x` is a runtime error, and an argument list cannot start with `~` (`println ~5` is a syntax error, `println (~5)` works). No ticket yet.
> - An index must be a number literal or a bare identifier: `a[n]` works, `a[n - 1]` is a syntax error. No ticket yet.
> - A function with no parameters can be written (`f: -> number`) but cannot be called, because a call needs at least one argument. No ticket yet.
> - Object parameter types do not work (`show: p Person -> string` leaves `p` unbound), and a parameter of function type (`g function`) is misread as a name. Return types of object type work. No ticket yet.
> - An object literal needs a trailing comma after a shorthand last field (`Person { name, }`).

### 3.3 Blocks and line ends

Blocks are decided by indentation, measured in columns.

- The **body of a function** is every following expression whose first token is in a column greater than the column of the function's name. The body of a **`for`** is every following expression whose first token is in a column greater than the column of the `for` keyword. A body ends at the first token that is not indented further.
- An expression ends at the end of its line, unless the line ends with an operator, a comma, `?`, `!`, or an opening `(`, `[` or `{`. Then it continues on the next line. A closing bracket may appear on a later line.
- A top-level expression that continues onto a new line must be indented past column 1.

```spk
total is 1 +
    2
```

## 4. Expressions and evaluation order

Evaluation is eager and strict, and runs strictly top to bottom through a file.

1. **Top level.** Expressions are evaluated one after another. The first error ends the program (see [section 9](#9-errors)).
2. **Literals** evaluate to themselves. An array literal evaluates its elements left to right.
3. **Binary operators** evaluate the left operand, then the right operand, then apply the operator. Both operands are always evaluated: `&` and `|` do not skip the right side. Whether they should is open ([Open questions](#13-open-questions), item 4).
4. **Calls** evaluate the arguments left to right, each completely before the next, then call the function. Arguments are passed by value for numbers, booleans and strings, and by shared reference for arrays and objects ([5.5](#55-references)).
5. **`if`** evaluates the condition, which must be a `bool`, then evaluates only the chosen arm. A missing arm yields `()`. (Today a string is also accepted as a condition, and an empty string counts as true; no ticket yet.)
6. **`for`** evaluates the iterable once, then runs the body once per element, in order.
7. **Assignment** `target is value` evaluates `value` first, then binds or updates `target` (see [section 6](#6-scoping-and-closures)).

Operand types not listed in the table below are a runtime error.

| Operator | Operands | Result |
| --- | --- | --- |
| `+` | number, number | sum |
| `+` | string, string | concatenation |
| `+` | array, array of the same element type | concatenation, a new array |
| `-` `*` | number, number | difference, product |
| `/` | number, number | float quotient. Division by zero is a runtime error. |
| `%` | number, number | remainder. A zero divisor is a runtime error. |
| `<` `>` | number, number, or string, string | `bool`; strings compare by code point |
| `=` | any two values of the same primitive type, or two `()` | `bool`; values of different types are never equal |
| `&` `\|` | bool, bool | logical and / or |
| `&` `\|` | whole numbers | bitwise and / or |
| `~` | number | negation |
| `~` | bool | logical not |
| `.` | object, field name | the field's value |

The result of `/` is always a float, even when both operands are integers: `6 / 3` is `2.0`, `7 / 2` is `3.5` (SPK-108). Mixing an integer and a float in `+ - * %` gives a float.

> **Current implementation.** There is one number type, so `6 / 3` prints `2`. `bool + bool` acts as "or" and `bool * bool` as "and"; these are not part of v0.2, which uses `|` and `&`. Equality of arrays and objects is always `false`; see [Open questions](#13-open-questions), item 5. `1 = 1.0` is `true`.

> **Current implementation.** Tail calls are meant to be optimized (the interpreter builds thunks for them), but in practice a self-recursive call in tail position still overflows the native stack: a 4,000-deep tail-recursive function aborted in a release build. See [Open questions](#13-open-questions), item 7.

Example (runs today):

```spk
a is 7
b is 2
println "{} / {} = {}" a b (a / b)
println "{}" (2 + 3 % 2 * 4)
```

```text
7 / 2 = 3.5
6
```

The same in Swahili (runs today with `SPEAK=sw`):

```spk
a ni 7
b ni 2
andika_laini "{} / {} = {}" a b (a / b)
```

```text
7 / 2 = 3.5
```

## 5. Values and types

A value has exactly one type. Speak is dynamically typed: variables have no declared type and can be given a value of any type. Types are declared only on function parameters and results ([7](#7-functions-and-type-checking)). There are no implicit conversions between types.

| Type | Written | Notes |
| --- | --- | --- |
| number | `42`, `3.14` | integer (`i64`) or float (`f64`), see 5.1 |
| bool | `true` / `false` (`kweli` / `ongo`) | |
| string | `"habari"` | immutable text |
| array | `[1, 2, 3]` | ordered, all elements of one type; type `[]T` |
| object | `Person { name, age, }` | named record of fields |
| function | `f: n number -> number` | first-class |
| empty | `()` | "no value" |

Planned in the roadmap and not specified here: bytes and map (SPK-108), error values (SPK-106, see [section 9](#9-errors)), resource handles (SPK-203), tasks (SPK-303).

### 5.1 Numbers

v0.2 has two numeric representations, `i64` integers and `f64` floats (SPK-108). A literal without a `.` is an integer; a literal with a `.` is a float. `/` always yields a float. The type word `number` accepts both; v0.2 declares no separate type word for integers (see [Open questions](#13-open-questions), item 2). Array indexes, slice bounds and the operands of bitwise `&` and `|` must be integers. Integer overflow is an error (item 2 of the open questions covers whether it wraps instead).

### 5.2 Strings

A string is a sequence of Unicode characters. `len` counts characters, and slicing is by character (SPK-109). `+` joins two strings.

> **Current implementation.** `len` counts UTF-8 bytes, so `len "héllo"` is 6. `for c in "abc"` yields each character cast to a byte (`97 98 99`), which is wrong for any non-ASCII text. Slicing and indexing a string are errors. SPK-109 fixes all three.

### 5.3 Arrays

An array literal `[a, b, c]` has the element type of its first element. An element of another type is a runtime error. `[]` is an empty array that fits any array type.

- `a[i]` reads element `i`, counted from 0. Reading past the end yields `()`.
- `a[i] is v` replaces element `i`. Writing past the end extends the array, filling the gap with `()`.
- `a[s..e]` is a new array of the elements from `s` up to but not including `e`. Either bound may be left out: `a[..e]`, `a[s..]`. A bound outside the array is a runtime error.
- `+` joins two arrays of the same element type into a new array.
- `len a` is the number of elements.

> **Current implementation.** A slice with a bound past the end panics the interpreter instead of raising a runtime error. No ticket yet. Assigning past the end can leave `()` inside a typed array.

```spk
numbers is [1, 2, 3, 10]
println "{}" (numbers[1])
println "{}" (numbers[9])
```

Runs today. The first line prints `2`; the second prints an empty line, because `numbers[9]` is `()`.

### 5.4 Objects

An object is a named record. `Person { name, age, }` builds an object of type `Person` whose fields are copied from the variables `name` and `age`. `p.age` reads a field, and `p.age is 30` replaces one. Reading or assigning a field that the object does not have is a runtime error. The type of an object is its name, and a function declared `-> Person` returns one.

```spk
name is "Osoro"
age is 23
p is Person {
    name,
    age,
}
p.age is 30
println "{} is {}" (p.name) (p.age)
```

Runs today and prints `Osoro is 30`. The Swahili version uses `ni`, `andika_laini`, and names of the learner's choice; `Mkoba { thamani, }` works the same way.

### 5.5 References

Numbers, booleans and strings are values. Arrays and objects are **shared references**: `b is a` makes `b` and `a` name the same array, so `b[0] is 99` is visible through `a`. Cycles (an object holding itself) are collected by a cycle collector (SPK-104).

> **Current implementation.** Arrays and objects are copied on assignment and on every call. After `copy is numbers` and `copy[0] is 99`, `numbers[0]` is unchanged. SPK-104 replaces copying with shared references.

### 5.6 Empty

`()` is the empty value. It is the value of an `if` with no matching arm, of a `for`, of `println`, and of an out-of-range index. Its type is the empty type. `()` is not a `bool`; using it as an `if` condition is a runtime error.

## 6. Scoping and closures

Scoping is **lexical** (SPK-103). The environment of a function is the one in which its definition was evaluated, not the one of its caller.

- **Scopes.** The top level of a file has one global scope. Each function call creates a new scope whose parent is the scope where the function was defined. A `for` loop creates a scope for its body, with the loop variable bound in it; its parent is the scope containing the loop. `if` arms do not create scopes.
- **Lookup.** A name is looked up in the current scope, then in each enclosing scope in turn. A name found nowhere is an error of the system category.
- **Binding and assignment.** `name is value` updates the nearest existing binding of `name` in the current or an enclosing scope. If there is none, it creates a binding in the current scope. Parameters and loop variables are bindings created fresh in their scope, so they can hide an outer name of the same spelling.
- **Function definitions.** `name: ... ` creates (or replaces) a binding for `name` in the current scope. Functions are ordinary values: they can be stored, passed and returned.
- **Closures.** A function value keeps its defining scope alive. Calls to it, from anywhere, read and update variables there. Two closures made in one scope share that scope's variables.
- **Builtins** are bindings in an outermost scope, so a program may shadow them.

Example (v0.2 only; does not run today):

```spk
make_counter: start number -> function
    count is start
    next: step number -> number
        count is count + step
        count
    next

c is make_counter 0
println "{}" (c 1)
println "{}" (c 1)
```

prints `1` then `2`. In Swahili:

```spk
tengeneza_kaunta: mwanzo nambari -> kazi
    jumla ni mwanzo
    ifuatayo: hatua nambari -> nambari
        jumla ni jumla + hatua
        jumla
    ifuatayo

k ni tengeneza_kaunta 0
andika_laini "{}" (k 1)
andika_laini "{}" (k 1)
```

prints `1` then `2`.

The difference between lexical and dynamic scoping shows when a parameter has the same name as a global. Under v0.2, `times` always sees the global `scale`:

```spk
scale is 10
times: n number -> number
    n * scale
run: scale number -> number
    times 2
println "{}" (run 3)
```

v0.2 prints `20`. The interpreter today prints `6`, because `times` sees the caller's `scale`. The Swahili version (`hesabu`, `mara`, `endesha`) behaves the same, printing `6` today.

> **Current implementation.** Scoping is dynamic: a call pushes its arguments onto the caller's frame chain, and a function value stores only its syntax. A nested function cannot read its parent's variables after the parent returns, so the counter example fails with "count is not defined". Closing the gap: SPK-103.

> **Current implementation.** Assigning to an outer variable inside a `for` body does not update it. In this program `count` stays `0`; it should print `3`:
>
> ```spk
> count is 0
> for i in [1, 2, 3]
>     count is count + 1
> println "{}" count
> ```
>
> The Swahili equivalent (`jumla`, `kwa i katika`) also prints `0`. The loop runs on a copy of the frame. SPK-103 removes the copy. The same happens inside functions, so a sum written with `for` inside a function returns its starting value.

## 7. Functions and type checking

### 7.1 Definitions and calls

```spk
name: param1, param2 type, param3 type -> result-type
    body
```

- Each parameter has a type. `a, b number` gives both `a` and `b` the type `number`.
- Parameter and result types are `number`, `bool`, `string`, an array type `[]T`, `function`, or the name of an object type.
- A call `name arg1 arg2` passes the arguments in order. The function is called with its own scope ([6](#6-scoping-and-closures)).
- Recursion is ordinary: the name is bound in the defining scope.
- Calling a non-function is a runtime error. Calling with the wrong number of arguments is a runtime error.

### 7.2 Result of a call

The body is a sequence of expressions evaluated in order. An expression whose value has the **declared result type** ends the call, and that value is the result. Expressions of any other type, including `()`, do not end it. This is how early exit works in Speak: an `if` with no matching arm yields `()` and the body goes on, and one that matches yields a value of the right type and the call ends. An assignment is skipped unless it is the last expression.

```spk
sign: n number -> string
    if n < 0 ? "negative"
    if n = 0 ? "zero"
    "positive"
println (sign 0)
println (sign 7)
```

Runs today and prints `zero` then `positive`. Swahili (`ishara`, `hasi`, `sifuri`, `chanya`, `-> mlolongo`) prints `sifuri` then `chanya`.

A value of the right type produced inside a `for` also ends the call, because the `for` yields it:

```spk
first_big: xs []number -> number
    for x in xs
        if x > 10 ? x
    0
println "{}" (first_big ([3, 12, 40]))
```

Runs today and prints `12`.

If no expression produces a value of the declared type, the call is a runtime error ("the expected return type `number` could not be constructed").

### 7.3 What is checked

An empty array `[]` matches any array parameter type.

| Check | When | Result |
| --- | --- | --- |
| Each argument matches its parameter type | at the call | runtime error naming the expected and found types and the argument position |
| The call produces a value of the declared result type | at the call | runtime error, see 7.2 |

A parameter of type `number` accepts integers and floats. Type errors are runtime errors, found when the line runs, not before the program starts.

```spk
f: n number -> number
    n
println (f "a")
```

Runs today and fails with `Runtime error: expected arg type of (number) but got (string), for 1 arg to (function literal 'f: number -> number')`.

> **Current implementation.**
> - The result check exists, but it is weaker than the README suggests: the first expression of the right type wins, even if it is not the last, so a body of `1` then `2` returns `1`, and a body of `"s"` then `n` in a `-> number` function returns `n`. SPK-110 tightens enforcement. Whether to keep "first expression of the declared type wins" is open ([Open questions](#13-open-questions), item 3).
> - Extra arguments are silently dropped, and missing ones leave the parameter unbound, which fails later with "is not defined". No ticket yet.
> - Recursion deeper than a few thousand calls (about 500 in a debug build) aborts the process with a native stack overflow. SPK-905 adds a recursion depth limit, so a runaway program fails with an error instead of aborting.

## 8. Builtins

| Locale key | English | Swahili | Behaviour |
| --- | --- | --- | --- |
| `builtins.print` | `print` | `andika` | Writes its text without a newline. |
| `builtins.println` | `println` | `andika_laini` | Writes its text and a newline. |
| `builtins.sprint` | `sprint` | `mlolongo_andika` | Returns the text as a string. |
| `builtins.len` | `len` | `urefu` | Length of a string or array. Any other argument is a runtime error. |
| `builtins.mod` | `mod` | `moduli` | Loads a module. Planned, see [section 10](#10-planned-modules). |

`print`, `println` and `sprint` take a template string first. Each `{}` in it is replaced by the string form of the next argument, in order. With no placeholders, the arguments after the first are appended. Numbers print without a trailing `.0` when whole. Booleans print as the locale's `true` and `false` words. The string form of arrays, objects and functions is for debugging and is not part of v0.2.

> **Current implementation.** `println "{} {}" 1` (too few arguments) panics the interpreter. `print` and `sprint` require an argument; a bare `println` or `len` evaluates to the function value. SPK-205 moves the builtins into a standard module `std/io` with a prelude that keeps beginner programs unchanged. SPK-406 adds string formatting.

## 9. Errors

Every error has a **category**, a **message** and a **position** `[line:column]` in the message.

| Category | Locale key | English | Swahili | Raised when |
| --- | --- | --- | --- | --- |
| syntax | `errors.const.syntax` | Syntax error | Hitilafu ya kisintaksia | the program does not parse; nothing runs |
| runtime | `errors.const.runtime` | Runtime error | Hitilafu ya wakati wa utekelezaji | a value or type is wrong: bad operand, wrong argument type, division by zero, non-bool condition |
| system | `errors.const.system` | System error | Hitilafu ya mfumo | an undefined name, or a problem outside the program |
| assert | `errors.const.assert` | Assertion error | Hitilafu ya kudai | the interpreter's own consistency check failed |

Messages come from the locale keys under `errors.*` and are written in the active locale. They are printed to standard error as `<category>: <message>`.

```spk
println "x"
println zzz
println "never"
```

Runs today. Output: `x`, then on standard error `System error: zzz is not defined [2:9]`. In Swahili, `andika_laini haipo` prints `Hitilafu ya mfumo: haipo haijafafanuliwa kwenye nafasi [2:14]`.

Syntax errors are found before the program runs. A runtime error stops the program at the line that raised it.

**v0.2 (SPK-106).** Errors are values. They carry a type, a message and a stack trace with `file:line`. Speak code can raise an error and catch one, using localized keywords. The keywords and their syntax are defined by SPK-106 and are not specified here. An error that nothing catches ends the program with a non-zero exit status and a trace. SPK-107 adds the offending source line with a caret under the column.

> **Current implementation.** An error is a category and a message. Nothing in Speak can catch it. The program stops, the message goes to standard error, and **the exit status is still 0** (`core/samples/error.spk` prints its error and exits 0). A few invalid programs panic the interpreter instead of raising an error (the empty string, a slice out of range, too few `{}` arguments, `mod`). SPK-106 fixes the exit status. The panics have no ticket yet.

## 10. Planned: modules

SPK-112 makes `mod` load a module: imports resolve file-relative paths, modules are cached, a module lists its explicit exports, and an import cycle is an error. The syntax is set by that ticket. SPK-801 and SPK-807 govern which locale a module is read in.

> **Current implementation.** `mod` panics (`unimplemented!`). The lexer produces a `::` token for module access, but the parser has no rule for it.

## 11. Planned: async and await

SPK-303 adds a task value and localized `async` and `await` keywords. Awaiting inside a function suspends it without blocking the event loop (SPK-302). The keywords' spelling in each locale and the exact syntax are decided there; this version reserves nothing. The decision is that they are keywords and not library functions.

## 12. Current implementation: summary of gaps

| Gap in today's interpreter | Closed by |
| --- | --- |
| Dynamic scoping, no closures | SPK-103 |
| `for` body cannot update outer variables | SPK-103 |
| Arrays and objects copied, not shared | SPK-104 |
| One `f64` number type; `/` of whole numbers prints `2`, not `2.0` | SPK-108 |
| `len` counts bytes; `for` over a string yields byte numbers; no string slicing | SPK-109 |
| Result type check is first-match, not last-expression | SPK-110 |
| Errors cannot be caught, exit status is 0, no stack trace | SPK-106 |
| No source line or caret in error output | SPK-107 |
| One process-wide locale | SPK-801 |
| Reserved words: only number, bool, string type words | SPK-807 |
| `mod` panics | SPK-112 |
| Builtins in the interpreter, not a std module | SPK-205 |
| Native stack overflow on deep recursion, no depth limit | SPK-905 (limit), SPK-103 |
| Tail calls not optimized | none |
| Empty string `""`, out-of-range slice and too few `{}` arguments panic | none |
| `&` `\|` do not chain or mix with comparisons; `~x` and `println ~5` fail | none |
| Index must be a literal or identifier | none |
| Zero-parameter functions cannot be called; object/function parameter types fail | none |
| Wrong argument count not an error | none |
| Identifiers ASCII only | none |
| A string used as an `if` condition is accepted, and an empty string counts as true | none |

## 13. Open questions

1. **Unicode identifiers.** Should identifiers allow any Unicode letter? Names in Amharic, Gĩkũyũ or Yoruba currently fail to lex, which cuts against the goal of coding in your own language. SPK-109 covers strings only.
2. **Number details (SPK-108).** Does integer overflow wrap or raise an error? Is there an integer type word beside `number`? Does `1 = 1.0` stay `true`? This spec assumes `number` covers both, overflow is an error, and `1 = 1.0` is `true`.
3. **Result of a call.** Keep "first expression of the declared type ends the call", or switch to "last expression is the result" with an explicit early exit? An explicit exit needs a new keyword, which the roadmap does not list. This spec keeps today's rule.
4. **Short-circuiting.** Should `&` and `|` skip the right operand when the left decides the result? This spec evaluates both.
5. **Equality of arrays and objects.** Today it is always `false`. With shared references (SPK-104) it could mean identity or contents.
6. **Wrong argument count.** This spec makes it an error. Partial application, which today half works, is not specified.
7. **Tail calls.** The roadmap says tail-call optimization should survive the rewrite, but the interpreter does not do it today. Is it a v0.2 requirement? It is cheap to promise and costly to add later.
8. **Assignment to an outer name.** `x is v` updates the nearest existing `x`, so a function cannot declare a new local that has the same name as a global (a parameter can). Is that what learners should see, or should assignment always create a local and need an explicit keyword to update an outer name? The counter closure needs the update behaviour, so this spec keeps it.
9. **Uncaught error exit status.** This spec attaches "exit non-zero" to SPK-106 because SPK-306 covers only tasks.
10. **Printing arrays and objects.** The default text of arrays and objects is the interpreter's debug format (`array ([]number): [Number(2.0), ...]`). It should be fixed before the conformance suite (SPK-102) records it as golden output.
11. **Tab and column rules.** A tab is one column today, which makes mixed tab and space blocks fragile.
