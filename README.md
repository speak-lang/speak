<div align="center">
	<img width="256" src="assets/logo.png" alt="Speak programming language logo">

# Speak programming language

Speak is an open-source programming language for learning to code in the language you think in.

</div>

Most programming tools assume English. Speak's keywords, builtins and error messages come from locale files, so learners can write code in their native tongue, not only in English. Speak supports English and Swahili today, and each language a community adds, such as Dholuo, Gĩkũyũ, Amharic or Yoruba, becomes one more language students can code in. It is built in the open with the communities it serves, and its first classroom pilot is in Kenya.

Speak is being built for classrooms with little or no internet: the standard library and lessons will ship inside the interpreter, and it will run offline on low-cost hardware such as a Raspberry Pi. The core is kept buildable for WebAssembly, so it can run in the browser and in mobile learning apps. Beginner programs stay simple; files, processes and networking will arrive as modules for learners ready to grow. See the [roadmap](docs/roadmap.md) for goals, milestones and design decisions.

It is expressive and dynamically typed (although all function signatures must be typed which becomes a runtime error if the types are violated by the function itself or its caller). A speak program is an expression.

## About

This language is directly inspired by Linus Lee's work on his language [ink](https://www.github.com/thesephist/ink) and his talk from GopherCon [here](https://www.youtube.com/watch?v=ALwmdcFiuGg&t=168s). Examples on how its intended to look like can be found in the samples directory. There's no formal specification yet, so far it's experimental. Writing one is the first ticket on the [roadmap](docs/roadmap.md) (SPK-101).

## Introduction

Below programs can run with the current interpreter implementation.

Here is are implementations of FizzBuzz in Speak.

English Speak, FizzBuzz.

```spk
for i in 1..15
    if i % 15 = 0
        println "FizzBuzz"
    else if i % 3 = 0
        println "Fizz"
    else if i % 5 = 0
        println "Buzz"
    else
        println i
```

Swahili Speak, FizzBuzz.

```spk
kwa i katika 1..15
    kama i % 15 = 0
        andika_laini "FizzBuzz"
    sivyo kama i % 3 = 0
        andika_laini "Fizz"
    sivyo kama i % 5 = 0
        andika_laini "Buzz"
    sivyo
        andika_laini i
```

English Speak, Fibonacci sequence.

```spk
// Fibonacci sequence generator
//
// naive implementation
fib: n number -> number
    if n = 0
        return 0
    if n = 1
        return 1
    return (fib n - 1) + (fib n - 2)

// memoized implementation
memo is [0, 1]
fibMemo: n number -> number
    if memo[n] = ()
        memo[n] is (fibMemo n - 1) + (fibMemo n - 2)
    return memo[n]

println "Naive solution: {}" (fib 20)
print "Dynamic solution: {}" (fibMemo 20)
```

Swahili Speak, Fibonacci sequence.

```spk
// utekelezaji jinga
fib: n nambari -> nambari
    kama n = 0
        rudisha 0
    kama n = 1
        rudisha 1
    rudisha (fib n - 1) + (fib n - 2)

// utekelezaji wa kumbukumbu
kumbukumbu ni [0,1]
fibKumbukumbu: n nambari -> nambari
    kama kumbukumbu[n] = ()
        kumbukumbu[n] ni (fibKumbukumbu n -1) + (fibKumbukumbu n - 2)
    rudisha kumbukumbu[n]

andika_laini "Matokeo ya utekelezaji jinga: {}" (fib 20)
andika "Matokeo ya utekelezaji wa kumbukumbu: {}" (fibKumbukumbu 20)
```

English Speak, Collatz sequence.

```spk
// finding long collatz sequences

next: n number -> number
    if n % 2 = 0
        return n / 2
    return 3 * n + 1

collatz: n number -> []number
    if n < 1
        return []
    if n = 1
        return [n]
    return [n] + collatz (next n) // arrays can be appended by AddOp

// run a search for longest collatz sequence under max
max is 1_000
longest is collatz max
print "Longest collatz seq under {} is {} items, sequence is {}", max, (len longest), longest
```

Swahili Speak, Collatz sequence.

```spk
// kutafuta mifuatano ya Collatz

inayofuata: n nambari -> nambari
    kama n % 2 = 0
        rudisha n / 2
    rudisha 3 * n + 1

collatz: n nambari -> []nambari
    kama n < 1
        rudisha []
    kama n = 1
        rudisha [n]
    rudisha [n] + collatz (inayofuata n) // safu zakusanywa kwa kutumia operesheni ya kuongeza

// tafuta mifuatano ya Collatz kwa hifadhi ya upeo
upeo ni 1_000
safu ni collatz upeo
andika "mifuatano ya Collatz refu zaidi kwa {} ni vitu {} vya safu, safu yenyewe ni {}", upeo, (urefu upeo), safu
```

## Getting Started

The interpreter can be found [here](https://github.com/muse254/speak/releases) to download and start using locally.

You can run Speak in 3 ways:

1. The Speak binary can be used to execute a Speak script. It can be run like so: `speak run main.spk`.
2. The Speak binary can initialize an interactive repl session where you can start typing Speak code. Initialized like so: `speak repl`.
3. Speak interpreter is written in Rust and Speak can be executed directly using the Rust interpreter's API.

## Building locally

To build locally, the [Rust toolchain](https://www.rust-lang.org/learn/get-started) must be installed.

Run:

```sh
make
```

The default interpreter assumes the English language. To run the Swahili variant, create a `SPEAK` environment variable and set it to `sw` before interactimg with the interprerter. For example:

```sh
SPEAK="sw" speak repl
```
