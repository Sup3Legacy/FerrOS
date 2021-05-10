---
title: FerrOS
author: Gabriel \textsc{Doriath Döhler}, Paul \textsc{Fournier}, Constantin \textsc{Gierczak-Galle}, Samuel \textsc{Vivien}
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

## Main objectives

When we first began working on FerrOS, we have fixed some objectives we wanted to achieve :

* General-purpose kernel with a working userland
* Some basic interactions using system calls
* Preemptive multitasking with a not-too-naive scheduling algorithm (basically at least something more advanced than a simple round-robin) and some notion of priority.
* As many accessible physical devices as possible (because that's fun)
* Multi-screen (i.e. the screen can be shared by multiple processes). The reason behind this is pretty simple : how would you better demonstrate your working multitasking system than by having multiple processes write on the display at the same time? :)
* Minimalist but functionnal userspace-library, which enables the user to write simple CLI programs.
* A small functionnal shell (how original).

This can be all summed up by "Have a working usermode shell that can do some basic stuff while a clock is running 'real-time' at the top of the screen".

# Programming

The very first stages of our kernel were written following Philip Oppermann's (TODO : add link) tutorial on writing a microkernel in Rust. This brought us to the point where we had a bootloader (TODO cf. plus loin), a primitive VGA teletype driver, a keyboard driver, page and memory allocation. Thanks to this tutorial, we could acquire the bases of low-level programming in Rust and could move far beyond this proof-of-concept kernel.

During this project, we used some crates instead of writing the code ourselves in areas where two conditions were met :

* A crate for that particularly use-case was available (was not often the case)
* The particular part of the kernel was tedious/uninteresting to write OR very technical, so that it would take us some time that we would prefer to spend on adding some more functionalities to the kernel

In practical, the main crates we used were :

* `x86-64` : a very useful crates with a lot of structures used in x86-64-system-programming. It provides safe wrapper to read and write from/to ports, manage GDT, paging tables, etc.
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

>The VFS is kind of the central piece of our kernel. It’s an energy field created by all living user-programs. It surrounds them and penetrates them. It binds the OS together [^1].

Like in a real UNIX-like kernel, every device, be it hardware or software, can be accessed by an user-program through the abstract high-level interface of the VFS. In our opinion, this is a good example of an occurence where Rust really shows its strength. Our VFS represents a really meaty chunk of code and contains of lot of layers of abstraction. 

The basic idea is the following : the VFS is an abstract tree whose branches are `String`s and whose leafs are various drivers. These drivers can be very different (we have a mouse driver, sound driver as well as a Tar driver, a RAM-disk driver, etc.) but every one of them implements the trait `Partition`, which presents a common abstract high-level interface, containing a few primitives (`read`, `write`, etc.). This is where we were really thankful of Rust's trait system, as we could unite a lot of drivers, which work completely differently under the hood (e.g. the driver for the Tar filesystem is over 1200-LOC long, contains multiple caches, a lot of data-structures and bytes handling all over the places, while the driver for the clock uses only some Port-based logic or the screen driver contains memory writes and buffer manipulations), in a single "simple" structure whithout worrying about any UB.

So, when the VFS receives a query from a program (or from the kernel), it follows a path on its `Partition`-tree according to the `Path` contained in the `OpenFileTable` associated with the `FileDescriptor` given in argument to the query. If it eventually reaches a `Partition` (that is a driver), it simply forwards the query to that driver and returns its result. (if the path ends within the tree, the VFS handles the query itself. For example, if a program from the path `/`, it gets `proc, hardware, ustar`, i.e. the names of repertories in the root-folder).

Given this structure, one can very simply add a new interface to the VFS, simply by adding a line into its initialization code (if the driver can be added at compile-time) or simply mutate the VFS main structure at runtime.

Each node in the tree holds a hash-map (we use Rust's `BTreeMap`) which associates a folder-name to the associated node. Being able to use such high-level data-structure even if we are building to a "bare-metal" target is really comfortable enabled us to build such complex structures in our kernel.


[^1]: Kernel Wars, A new filesystem

## Program/kernel interaction

A program cannot by itself interact with the user or with the underlying hardware, as all software and hardware ressources are managed by the kernel (our drivers are all part of the kernel space for simplicity sakes). Every interaction between a program and the kernel is done via either a forced context switch or a software interrupt, implementing a syscall.

When a program requires a ressource from the kernel, it generates a syscall, whose number corresponds to a pre-defined list of possible syscalls

## User-space

### Librust

#### General philosophy

The user-space is a very important part of any OS and it comes through a flexible enough OS-specific library (often called `libc`). As we wished ourselves a pure-Rust OS (+- some ASM of course), we simply could not make (or copy from a *Nix OS) a `libc`. It has been replaced by a `librust`.

Our initial plan was to properly cross-compile the Rust compiler by adding a custom target (that is the FerrOS target), as well as all low-level bindings. Usually, the Rust std-lib is meant to get bound to the OS' `libc`. In our case, we would have directly build the low-level library inside the std-lib.

However, this plan turned out to be quite a lot more work (be it purely implementation of the library or simply research in order to first understand what's going on in the gigantic mess that is the Rust std-lib) than we could do in just a few months (we also had to build an entire kernel besides that!), so we made a compromise : we built a `librust` that is a standalone `no-std` crate containing all low-level bindings to the OS (syscalls, etc.) as well as some abstraction layers on top of that. This crate can be imported when building a user-program for our OS.

#### Main modules

The `librust` contains a few main modules that helps us build software for our target with minimal effort and maximal possibilities

##### Kernel low-level interaction

One of the most important module is the one containing all the very-low-level code responsible for all interactions with the kernel, through the syscalls. It only contains a few lines of inline-ASM and has been tested to ensure there as little risk of register/memory corruption.

On top of this code are build a few abstraction layers for easy handling of files, I/O data, etc. We decided to not go as overkill as the std-lib regarding this abstraction, as we did not have a lot of time, and because ouf interactions are a lot simpler than most *Nix systems, so there is no need for such very-high-level abstraction.

##### Memory allocator

A very useful primitive traditionaly offered by a system's `libc` is, of course, `malloc`. We have our own heap allocator (it is based on Phil Opp.'s design, that we improved a lot TODO expliquer où).

When a program first executes, it is given as arguments the start address and the size of the heap is has been allocated by the kernel. The allocator is then initialized with these values.

This allocator is just a simple linked_list allocator. We improved it quite a lot by :

- adding the automatic merging of contigous regions. Without it, a program could "run out" of heap space simply besauce of repeated allocations that fragmented the heap.
- adding the automatic request of page allcoation. In case the allocator could not allocate a heap region because no big enough region was left, it would request, through a syscall, the kernel some additional pages that it would (depending of if the kernel responded positively or not) itnegrate into the heap.

# Drivers

In order to be able to interract with various hardware alemetns, we wrote a couple basic drivers.

## VGA

The most important driver in our system is obviously the screen! We opted for a standard text-based interface, as we didn't want to mess with video-mode and pixel-fonts (we made a few tests of video-mode and concluded we would not need the capabilities of a video-based display).

For the display, we have multiple stacked interfaces. The most low-level one, in [`vga/mainscreen.rs`](../../src/vga/mainscreen.rs), handles all the basic logic associated  with a text-based terminal : it contains a reference to a fixed-sized buffer located at `0xb8000` and writes the individual characters into that buffer.

As we wanted our system to be "multitasking-ready™", we needed some form of multi-screen abstraction layer. That is what [`vga/virtual_screen.rs`](../../src/vga/virtual_screen.rs) is for. The basic idea is the following : each process is given a `VirtualScreen`, instanciated with a null size. It is free (obviously, in a real system this would require some sort of permissions) to change its size ans location on the `MainScreen`. It can write into that `VirtualScreen` transparently, as if it was a free-sized physical screen (this is done through the `VFS` and take into account some special characters to change color, cursor location, etc.), even though a `VirtualScrene` is simply an abstract structure containing a buffer. Each `VirtualScreen` is associated a `Layer`, which determines whether it must be displayed on top of other screens it may collide with.

The `MainScreen` (instantiated at startup) contains all `VirtualScreen`s in a priority-queue, indexed by layer. Whenever a `VirtualScreen` is written into, the `MainScreen` gets updated. It simply loops through all `VirtualScreen`s by order of increasing `Layer` and copies them into its buffer, at the given location and size.

This way, we can have multiple process display text onto the screen concurrently, sharing it and/or overlapping one another while preserving a unified interface for the screen : from within a process, we do not need to worry about the offset of the `VirtualScreen`, we simply write into it as if we could use the entire screen! 

There is however a small catch : as mentionned, the size and location (location on the physical screen) of a process' `VirtualScreen` can be changed by that process, which could potentially cause some issues. If we had more time, we would have implemented some sort of basic window managment system, like a simplified `i3`. Being aware of this limitation, we are still pretty proud of that part of the project, as it seems to us really nice being able to edit some text while a clock is asynchronously displaying the time on the top part of the screen!

## Keyboard and mouse

The other important driver we wrote is the keyboard one! There is nothing really special : we activated the corresponding interrupt and got some structures setup containing a `VecDeque` to store the incoming scancodes from the keyboard. Whenever a program reads bytes from `/hardware/keyboard`, these scancodes are poped from the queue and handed over to the process. We also implemented the same structure for the mouse, which can be accessed through `/hardware/mouse`, even though we currently have no user-program using the mouse!

## Sound

As we wanted to be able to offer the user a complete experience, we needed some sort of basic sound driver. We therefore use the integrated beeper as an 70s-style single-voice speaker. To be able to take full advantage of that horrible-sounding beeper, we wrote a "complex" interface build around a priority-queue. Each sound event is encoded into a `SoundElement`, containign some information, like its tone, length and time-offset (from current time) it should start on. This way, we can provide the driver with a series of noted each with an offset, in order to play some music without worrying about the program accessing the speaker ar the right time for the next note. 

The priority-queue uses as priority a tuple `(time_to_start, sound_id)` so that the sound poped from the queue is always the one that the driver needs to start playing. This also handles the situation of having multiple sound overlap (e.g. sound A starts at `1` and lasts for `2` ticks and B starts at `2` and lasts for `2` ticks : A plays from `1` to `3` and B from `3` to `4`.)

We simply keep track of the time by having a simple dummy variable incremented at each timer-interrupt (because the sound-driver updates its state at each such interrupt).

This driver provides a simple interface : [sound/mod.rs](../../src/sound/mod.rs).

There are some things we could have done if we had more time :

* add a `repeat` fild into `SoundElement` so that one can get a repeating sound.
* move from a beeper-based driver to a more advanced one using buffers, etc. This way, we could have provided a mroe complex interface using a number of different voices.

## Clock

We have a straight-forward (using OSdev) driver that reads the CMOS RTC and outputs the time informations. It can be accesses by the kernel or a program through `/hardware/clock`.

## UsTar

We really wanted to have a way to load and store data in a persistent way, using Qemu's disk emulation. Therefore we had to choose a filesystem format, so we took whatever the most simple one was : Tar. It went through some modifications and simplifications as we didn't need all features the original Tar format offers.
