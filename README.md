<div align="center">
	<img width="256" src="assets/logo.png" alt="Ink programming language logo">

# Speak programming language

Speak is an experimental programming language that aims to have configurable keywords. This is in an attempt to allow for the use of multiple written languages as preferred by the programmer.

</div>

It's main goal is for it to be used as an educational tool for non-english speaking minorities. This can happen using a configuration file that maps these translations to build an interpreter variant. We can have an English variant of Speak, a Swahili variant; and so on.

It's not meant to do any "heavy lifting" tasks like interacting with networks, writing servers... but once it's stable we can expose these primitives through Rust native functions. At the moment, they are non goals.

It is expressive and dynamically typed (although all function signatures must be typed which is a runtime error if the types are violated by the function itself or its caller). A speak program is an expression.

## About

This language is directly inspired by Linus Lee's work on his language [ink](https://www.github.com/thesephist/ink) and his talk from GopherCon [here](https://www.youtube.com/watch?v=ALwmdcFiuGg&t=168s). Examples on how its intended to look like can be found in the samples directory. There's no formal specification yet, so far it's experimental.

## Introduction
Below programs can run with the current interpreter implementation.

Here is are implementations of FizzBuzz in Speak.

With default `English` configuration.
```spk
fizzbuzz: n number -> string
    if n % 15 = 0 ? "FizzBuzz"
    if n % 3 = 0 ? "Fizz"
    if n % 5 = 0 ? "Buzz"
    sprint n

// call with 100
printf "fizzbuzz result for {} is {}" 100 (fizzbuzz 100)
printf "fizzbuzz result for {} is {} itself" 7 (fizzbuzz 7)
```

In Swahili
```spk
fizzbuzz: n nambari -> mlolongo
    kama n % 15 = 0 ? "FizzBuzz"
    kama n % 3 = 0 ? "Fizz"
    kama n % 5 = 0 ? "Buzz"
    mlolongo_andika n

// ita na nambari 100
andika_umbizo "matokeo ya fizzbuzz kwa {} ni {}" 100 (fizzbuzz 100)
andika_umbizo "matokeo ya fizzbuzz kwa {} ni {} yenyewe" 7 (fizzbuzz 7)
```

English Speak, Fibonacci sequence.
```spk
// naive implementation
fib: n number -> number
    if n = 0 ? 0
    if n = 1 ? 1
    (fib n - 1) + (fib n - 2)

// memoized implementation
fibMemo: n number -> number
    memo is [0, 1] 
    if memo[n] = () ? memo[n] is (fibMemo n - 1) + (fibMemo n - 2)
    memo[n]

printf "Naive solution: {}" (fib 25)
printf "Dynamic solution: {}" (fibMemo 20)
```


Swahili Speak, Fibonacci sequence.
```spk
// utekelezaji jinga
fib: n nambari -> nambari
    kama n = 0 ? 0
    kama n = 1 ? 1
    (fib n - 1) + (fib n - 2)

// utekelezaji wa kumbukumbu
fibKumbukumbu: n nambari -> nambari
    kumbukumbu ni [0,1]
    kama kumbukumbu[n] = () ? kumbukumbu[n] ni (fibKumbukumbu n -1) + (fibKumbukumbu n + 1)
    kumbukumbu[n]

andika_umbizo "Matokeo ya utekelezaji jinga: {}" (fib 25)
andika_umbizo "Matokeo ya utekelezaji wa kumbukumbu: {}" (fibKumbukumbu 20)
```


English Speak, Collatz sequence.
```spk
// finding long collatz sequences

next: n number -> number
    if n % 2 = 0 ? n / 2 ! 3 * n + 1

collatz: n number -> []number
    if n < 1 ? []
    if n = 1 ? [n]
    [n] + collatz (next n) // arrays can be appended by AddOp

// run a search for longest collatz sequence under max
max is 1_000
longest is collatz max
printf "Longest collatz seq under {} is {} items, sequence is {}", max, (len longest), longest
```

Swahili Speak, Collatz sequence.
```spk
// kutafuta mifuatano ya Collatz

inayofuata: n nambari -> nambari
    kama n % 2 = 0 ? n / 2 ! 3 * n + 1

collatz: n nambari -> []nambari
    kama n < 1 ? []
    kama n = 1 ? [n]
    [n] + collatz (inayofuata n) // safu zakusanywa kwa kutumia operesheni ya kuongeza

// tafuta mifuatano ya Collatz kwa hifadhi ya upeo
upeo ni 1_000
safu ni collatz upeo
andika_umbizo "mifuatano ya Collatz refu zaidi kwa {} ni vitu {} vya safu, safu yenyewe ni {}", upeo, (urefu upeo), safu
```

## Getting Started
The interpreter can be found [here](https://github.com/muse254/speak/releases) to download and start using locally.

You can run Speak in 3 ways:
1. The Speak binary can be used to execute a Speak script. It can be run like so: `speak run main.spk`.
2. The Speak binary can initialize an interactive repl session where you can start typing Speak code. Initialized like so: `speak repl`.
3. Speak interpreter is written in Rust and it Speak can be execute directly using the Rust interpreter's API.

