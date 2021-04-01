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

Some other crates brought us some convenient structures and macros but can be considered as "accessories".



# Practical characteristics



## Scheduler

Here, we discuss the implementation we opted for our scheduler. As of now, it is a simple preemptive, lottery-based, single-core scheduler, though the concept could be adapted to the context of multi-core CPUs. 

### When is the scheduler used?

Whenever there is an interruption that pauses the execution of a process (clock, halt, normal end of a process, etc.), the OS safely saves all the context of the process (that is its registers, flags, and memory pages), asks the scheduler what to do, and then loads up the context of the selected process and gives it control back.

### What scheduling schema is implemented?

We opted for a lottery-based scheduler that is able to implement priories. The process are divided in several groups, one for each priority, and the scheduler randomly selects a priority. We have chosen 8 priorities, so the scheduler pick a random byte (*hopefully* selected uniformly), and the most significant non null bit gives the priority that should be ran (it means that priority $p$ has twice more chances of being ran than priority $p+1$). The choice among a same priority is simply a round-robin. If however, the selected priority $p$ is empty, then it is upgraded to run the priority $p-1$, and so on. A problem would arise if there was no process having a priority $\le p$, but it suffices to put a dummy process that does nothing but give the control back to the scheduler at priority $0$ to solve this issue.

### Why this choice?

One of the main reasons is that it is a straightforward scheduling algorithm. It mostly consists of a 8-bit pseudo-random number generator, and a little boiler plate to link everything together. However, it still allows to implement an abstraction of the scheduling, so it is easy to change the algorithm in the future if needed. What's more, it also allowed us to use the clock interruptions and get a better grasp on how process scheduling and, in particular, context switch, worked.

## Virtual File system

The VFS is kind of the central piece of our kernel. It’s an energy field created by all living user-programs. It surrounds them and penetrates them. It binds the OS together.

Like in a real UNIX-like kernel, every device, be it hardware or software, can be accessed by an user-program through the abstract high-level interface of the VFS.

## Program/kernel interaction

A program cannot by itself interact with the user or with the underlying hardware, as all software and hardware ressources are managed by the kernel (our drivers are all part of the kernel space for simplicity sakes). Every interaction between a program and the kernel is done via either a forced context switch or a software interrupt, implementing a syscall.

When a program requires a ressource from the kernel, it generates a syscall, whose number corresponds to a pre-defined list of possible syscalls