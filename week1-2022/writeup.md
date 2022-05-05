# Writeups for week1 in 2022

## Program1 - Uppercase

## Manual

* off-by-one bugs found in allocating buffers for argv[1], which ends with null pointers.
* off-by-one is able to completely comprise a program, but don't know how.

## Static

* cland-tidy didn't report any bugs.
* clang-tidy needs compilation-base to help the analysis, especially in large program.

## Dynamic

### Valgrind

* fail to install valgrind on [m1 macos](https://github.com/LouisBrunner/valgrind-macos/issues/47)
* test on Ubuntu-18.04 intel machine, shows only the heap memory usage and detect possible leak, and reports no bug

```bash
ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 0 from 0)
```

### Sanitizers

* fail to add some `fsanitize` flags using the built-in MacOS clang

```bash
clang: error: unsupported option '-fsanitize=leak' for target 'arm64-apple-darwin21.4.0'
```

* on MacOS, when enable the `fsanitize` flag in command line, it will automatically generate `.dSYM` debug folders, while not in Makefile build.

* test on Ubuntu-18.04 intel machine, reports a bug with context:

```bash
ERROR: AddressSanitizer: dynamic-stack-buffer-overflow on address 0x7fffffffe0e5 at pc 0x000000512285 bp 0x7fffffffe020 sp 0x7fffffffe018
```

* use pattern-matching to build multiple files.
* add `-fsanitize=address...` to compile variables `CFLAGS`, `CXXFLAGS` and `LDFLAGS` in makefile when in debug mode to auto enable memory-bug detection, while remove it in release mode or performance testing.

## Program2 - Linked Lists

### Manual

two memory leaks:

* forget to free the 10th node in `swap_tenth_node`
* didn't free the last node at the end of program, should set while condition to `curr != NULL`

### Static

* a FP of null pointer de-ref, supposed that variable kNumElements is 0.
* add conse to kNumElements, the FP disappear

### Dynamic

* use valgrind, reports one leak:

```bash
still reachable: 32 bytes in 1 blocks
```

* build with sanitizers, reports two leaks, 32 bytes means two nodes are leaked

```bash
SUMMARY: AddressSanitizer: 32 byte(s) leaked in 2 allocation(s).
```

## Program3 - Parsing and Early Returns

### Manual

* potential memory leak when finding the close bracket, not handle error case properly

### Static

* run with clang-tidy, reports an warning:

```bash
/Users/chenxiang/Learn/CS110L/cs110l-spr-2020-starter-code/week1-2022/3-bracket-parser.c:33:9: warning: Potential leak of memory pointed to by 'mutable_copy' [clang-analyzer-unix.Malloc]
        printf("Malformed input!\n");
```

### Dynamic

* build with sanitizers, reports leak when input without close bracket, 3 bytes is the length of `mutable_copy`.

```bash
 ubuntu  ~/Reverse  ./3-bracket-parser "[1"
Malformed input!

=================================================================
==11828==ERROR: LeakSanitizer: detected memory leaks

Direct leak of 3 byte(s) in 1 object(s) allocated from:
    #0 0x4363f0  (/home/ubuntu/Reverse/3-bracket-parser+0x4363f0)
    #1 0x51211b  (/home/ubuntu/Reverse/3-bracket-parser+0x51211b)
    #2 0x512501  (/home/ubuntu/Reverse/3-bracket-parser+0x512501)
    #3 0x7f7bc79cec86  (/lib/x86_64-linux-gnu/libc.so.6+0x21c86)

SUMMARY: AddressSanitizer: 3 byte(s) leaked in 1 allocation(s).
```

### LibFuzzer

Use the brew-installed clang to enable LibFuzzer, while not functions well, the fuzzing program seems not terminating.

```bash
/opt/homebrew/opt/llvm/bin/clang -g -O0 -Wall -Wextra -std=gnu99 -fsanitize=fuzzer,address,leak -o 3-bracket-parser.fuzz 3-bracket-parser.c
```

test on Ubuntu-18.04 intel machine using the same build command, reports a leak in seconds:

```bash
==12187==ERROR: LeakSanitizer: detected memory leaks

Direct leak of 66 byte(s) in 1 object(s) allocated from:
    #0 0x46e900  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x46e900)
    #1 0x54a691  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x54a691)
    #2 0x54ae14  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x54ae14)
    #3 0x42e9f7  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x42e9f7)
    #4 0x439264  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x439264)
    #5 0x43a8cf  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x43a8cf)
    #6 0x429c8c  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x429c8c)
    #7 0x41cb52  (/home/ubuntu/Reverse/3-bracket-parser.fuzz+0x41cb52)
    #8 0x7fc6f5e21c86  (/lib/x86_64-linux-gnu/libc.so.6+0x21c86)

SUMMARY: AddressSanitizer: 66 byte(s) leaked in 1 allocation(s).
INFO: to ignore leaks on libFuzzer side use -detect_leaks=0.

MS: 1 InsertRepeatedBytes-; base unit: 44f4b07071a6ab0235666745b445670da6c18abe
artifact_prefix='./'; Test unit written to ./leak-713a3089b600714ac041f1fbaaf7ff619be1d74e
 ubuntu  ~/Reverse  cat ./leak-713a3089b600714ac041f1fbaaf7ff619be1d74e
X[[[[[[[[[[[VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV�������������������������������������%
```

## Program4 - Fibonacci

### Manual

* A compiler flag tips:
  * CFLAGS(C Compiler), CXXFLAGS(C++ Compiler) is specific, used when compiling and linking C/C++ programs.
  * CPPFLAGS(C PreProcesser) is universal.

* Fibonacci sequence should start at 0, so fix the base value.

### With Tools

* test with clang-tidy and sanitizers, reports no bugs, because it's a symantic/logical bug which is hard to detect.
