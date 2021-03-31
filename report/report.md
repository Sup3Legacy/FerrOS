---
title: FerrOS
author: Gabriel \textsc{Doriath-Döhler}, Paul \textsc{Fournier}, Constantin \textsc{Gierczak-Galle}, Samuel \textsc{Vivien}
abstract: FerrOS is a UNIX-like OS based on a minimalist hybrid-approach-kernel. Its main particularity is the language it is written it. FerrOS is written in pure Rust.
toc: true
numbersections: true
---

# Motivation

## Language

One of the first considerations that had to be made for this project was the language. We chose to use the Rust language, as using a language with more abstraction capabilities than the commonly used C language could be interesting when writing an OS, as abstracting some structures and methods would be beneficial in terms of ease of development.

We were aware that this language, however, has a lot less documentations when it comes to system programing, as it is very young (the first stable version was released only 6 years ago) and only a few such projects have been written using it.

We still managed to find enough documentation regarding the language-specific settings we would need (ranging from the `asm!` macro to the configuration of a `no-std` bareback target) and could adapt all the technical documentation that is explained using the C programming language in most topic-specific resources (such as OSdev).



# Theoretical plan

Before talking about how our kernel works, let's first explain how we designed it.
## General idea

To give a rough idea, our goal during this project was to write a sort of UNIX-like kernel (though we never designed our kernel as being really compatible with any *NIX OS). So the base idea isn't original at all. However, the choice of the language, as we thought, would make this project interesting, as OS written in Rust still are not common. One example would be Redox (TODO insert link), the best known and probably most advanced operating system written in Rust. 


# Programming

The very first stages of our kernel were written following Philip Oppermann's (TODO : add link) tutorial on writing a microkernel in Rust. This brought us to the point where we had a bootloader (TODO cf. plus loin), a primitive VGA teletype driver, a keyboard driver, page and memory allocation. Thanks to this tutorial, we could acquire the bases of low-level programming in Rust and could move far beyond this proof-of-concept kernel.

During this project, we used some crates instead of writing the code ourselves in areas where two conditions were met :

* A crate for that particularly use-case was available (was not often the case)
* The particular part of the kernel was tedious/uninteresting to write OR very technical, so that it would take us some time that we would prefer to spend on adding some more functionalities to the kernel

In practical, the main crates we used were :

* `x86-64` : a very useful crates with a lot of structures used in x86-64-system-programming. It provides safe wrapper to read and write from/to ports, manage IDT, GDT, paging tables, etc.
* `Bootloader` : a bootloader is something we didn't want to write at the beginning of the project. TODO on en fait un maintenant?
* `xmas-elf` : an ELF parsing utility. This is something we could have written ourselves but it involves a lot of boilerplate code that would take a long time to write.

Some other crates brought us some convinient structures and macros but can be considered as "accessories".



# Practical characteristics