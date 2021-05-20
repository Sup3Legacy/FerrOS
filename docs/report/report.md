---
title: FerrOS
author: Gabriel \textsc{Doriath Döhler}, Paul \textsc{Fournier}, Constantin \textsc{Gierczak-Galle}, Samuel \textsc{Vivien}
abstract: FerrOS is a UNIX-like OS based on a minimalist hybrid-kernel. Its main particularity is the language it is written it. FerrOS is written in pure Rust.
toc: true
numbersections: true
---

# Preliminaries

Any reader is strongly encouraged to have the code of the kernel available when reading this report. Rust also provides a **very** well-done documentation tool. The command `make doc` at the root of the repo should build the documentation and open it in the browser. We have put some effort into making a documentation clear and explicit!

# Motivation

## Language

One of the first considerations that had to be made for this project was the language. We chose to use the Rust language, as using a language with more abstraction capabilities than the commonly used C language could be interesting when writing an OS, as abstracting some structures and methods would be beneficial in terms of ease of development.

When we were first thinking about that language, someone (who asked to remain anonymous) told us :

> But why would you want to use Rust instead of the all-mighty C? Rust's "safeness" comes with a great amount of limitations and those who give up their liberty for the sake of winning some temporary safety get neith#(IAç]/l5Q¦Bmçtl¿(Fx **Segmentation fault (core dumped)**

This sums up pretty well the pros and cons of Rust. A very advanced and experienced user could do certain things a lot simpler using C because they wouldn't have to worry about data lifetime, cursed memory mutations and data races detection. But they would probably still encounter more segfaults than we did when developing the kernel. Aside from the development of the bootloader (raw ASM, so very prompt to crashes) and memory/page allocator, we really encountered a very reasonable amount of segfaults and pagefaults, and all of them were caused by mistakes in our page allocating routines. This meant that, when we had all the very technical bases in place, we almost didn't have to worry about any crash.

We were aware that this language, however, has a lot less documentations when it comes to system programing, as it is very young (the first stable version was released only 6 years ago) and only a few such projects have been written using it.

We still managed to find enough documentation regarding the language-specific settings we would need (ranging from the `asm!` macro to the configuration of a `no-std` baremetal target) and could adapt all the technical documentation that is explained using the C programming language in most topic-specific resources (such as OSdev).

### Summary of the main features of Rust
- Memory safety.
- Algebraic data types with `enum`.
- Powerfull type system with traits.
- Reproducible build with `Cargo.lock`.
- Macros: DRY (Don't  Repeat Yourself). This allowed to significantly reduce the syze of some programs (see the branch `keyboard` or `FerrOS-user/lisp` or `FerrOS-user/music`, ...).
- Conditional compilation. This is usefull for automatic testing (see the branch `testing`) without launching the graphical interface.

### Some drawbacks of using Rust
- We wanted to code a lisp parser and interpreter for FerrOS.
But we got a bit carried away with Rust's advanced features.
We created macros to implement parser combinators.
Sadly, it was getting to difficult to manage all of the lifetime parameters so we had to abandon the project.
- We had to use Rust unstable (Nightly) so stability is a bit of an issue.
- Nix (for even more reproducile builds) doesn't work well will Rust Nightly and cross compilation.

# Theoretical plan

Before talking about how our kernel works, let's first explain how we designed it.

## General idea

To give a rough idea, our goal during this project was to write a sort of UNIX-like kernel (though we never designed our kernel as being really compatible with any *NIX OS). So the base idea isn't original at all. However, the choice of the language, as we thought, would make this project interesting, as OS written in Rust still are not common. One example would be [Redox](https://www.redox-os.org/), the best known and probably most advanced operating system written in Rust.

## Main objectives

When we first began working on FerrOS, we have fixed some objectives we wanted to achieve :

* General-purpose kernel with a working user space
* Some basic interactions using system calls
* Preemptive multitasking with a not-too-naive scheduling algorithm (basically at least something more advanced than a simple round-robin) and some notion of priority.
* As many accessible physical devices as possible (because that's fun)
* Multi-screen (i.e. the screen can be shared by multiple processes). The reason behind this is pretty simple : how would you better demonstrate your working multitasking system than by having multiple processes write on the display at the same time? :)
* Minimalist but functional user space-library, which enables the user to write simple CLI programs.
* A small functional shell (how original).

This can be all summed up by "Have a working user mode shell that can do some basic stuff while a clock is running 'real-time' at the top of the screen".

# Programming

The very first stages of our kernel were written following [Philip Oppermann's tutorial](https://os.phil-opp.com/) on writing a microkernel in Rust. This brought us to the point where we had a bootloader, a primitive VGA teletype driver, a keyboard driver, page and memory allocation. Thanks to this tutorial, we could acquire the bases of low-level programming in Rust and could move far beyond this proof-of-concept kernel.

During this project, we used some crates instead of writing the code ourselves in areas where two conditions were met :

* A crate for that particularly use-case was available (was not often the case)
* The particular part of the kernel was tedious/uninteresting to write OR very technical, so that it would take us some time that we would prefer to spend on adding some more functionalities to the kernel

In practical, the main crates we used were :

* `x86-64` : a very useful crates with a lot of structures used in x86-64-system-programming. It provides safe wrapper to read and write from/to ports, manage GDT, paging tables, etc.
* `Bootloader` : a bootloader is something we didn't want to write at the beginning of the project. We implemented part of a bootloader to start the boot sequence.
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

Like in a real UNIX-like kernel, every device, be it hardware or software, can be accessed by an user-program through the abstract high-level interface of the VFS. In our opinion, this is a good example of an occurrence where Rust really shows its strength. Our VFS represents a really meaty chunk of code and contains of lot of layers of abstraction. 

The basic idea is the following : the VFS is an abstract tree whose branches are `String`s and whose leafs are various drivers. These drivers can be very different (we have a mouse driver, sound driver as well as a Tar driver, a RAM-disk driver, etc.) but every one of them implements the trait `Partition`, which presents a common abstract high-level interface, containing a few primitives (`read`, `write`, etc.). This is where we were really thankful of Rust's trait system, as we could unite a lot of drivers, which work completely differently under the hood (e.g. the driver for the Tar file system is over 1200-LOC long, contains multiple caches, a lot of data-structures and bytes handling all over the places, while the driver for the clock uses only some Port-based logic or the screen driver contains memory writes and buffer manipulations), in a single "simple" structure without worrying about any UB.

So, when the VFS receives a query from a program (or from the kernel), it follows a path on its `Partition`-tree according to the `Path` contained in the `OpenFileTable` associated with the `FileDescriptor` given in argument to the query. If it eventually reaches a `Partition` (that is a driver), it simply forwards the query to that driver and returns its result. (if the path ends within the tree, the VFS handles the query itself. For example, if a program from the path `/`, it gets `proc, hardware, ustar`, i.e. the names of repertories in the root-folder).

Given this structure, one can very simply add a new interface to the VFS, simply by adding a line into its initialization code (if the driver can be added at compile-time) or simply mutate the VFS main structure at runtime.

Each node in the tree holds a hash-map (we use Rust's `BTreeMap`) which associates a folder-name to the associated node. Being able to use such high-level data-structure even if we are building to a "bare-metal" target is really comfortable enabled us to build such complex structures in our kernel.


[^1]: Kernel Wars, A new filesystem

## Virtual memory

Virtual memory is used to have real processes with separated space. We haven't activated ring 3 due to lack of time to understand the error we got when trying to activate it but the whole paging has been done considering the user program would be in ring 3 (such as using the `USER_ACCESSIBLE` flag).

In all, every process has it's own level 4 table and the kernel is a higher space kernel with with the level 3 tables of the kernel in common between every process.

In terms of performances and space used. The frame allocator has a array of booleans to store wich pages of 4KiB are available to be allocated.

## Program/kernel interaction

A program cannot by itself interact with the user or with the underlying hardware, as all software and hardware resources are managed by the kernel (our drivers are all part of the kernel space for simplicity sakes). Every interaction between a program and the kernel is done via either a forced context switch or a software interrupt, implementing a syscall.

When a program requires a resource from the kernel, it generates a syscall, whose number corresponds to a pre-defined list of possible syscalls

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

On top of this code are built a few abstraction layers for easy handling of files, I/O data, etc. We decided to not go as overkill as the std-lib regarding this abstraction, as we did not have a lot of time, and because our interactions are a lot simpler than most *Nix systems, so there is no need for such very-high-level abstraction.

##### Memory allocator

A very useful primitive traditionally offered by a system's `libc` is, of course, `malloc`. We have our own heap allocator (it is based on [Phil Opp.'s design](https://os.phil-opp.com/), that we improved a lot).

When a program first executes, it is given as arguments the start address and the size of the heap is has been allocated by the kernel. The allocator is then initialized with these values.

This allocator is just a simple linked_list allocator. We improved it quite a lot by :

- adding the automatic merging of contiguous regions. Without it, a program could "run out" of heap space simply besauce of repeated allocations that fragmented the heap.
- adding the automatic request of page allocation. In case the allocator could not allocate a heap region because no big enough region was left, it would request, through a syscall, the kernel some additional pages that it would (depending of if the kernel responded positively or not) integrate into the heap.

### User applications

Of course, an operating system is not complete without its user suite. We, given the time constraint, cherry-picked the most fundamental and useful utilities one need. Here is a list of those implemented, and a short description.

#### Utilities

- `shell`, a command interpret, capable of some file descriptor manipulation (`|`, `>`, `>>`, `<`, `<<`, `&`)

The syntax is exactly the same as the one from the `mini-shell` TP.

- `cat`, to read the content of a file or list the elements of a folder (replaces the traditional `ls`)
- `hexdump`, to dump the byte sequence of a file
- `echo` to redirect an input (originally `STD_IN`) to an output (originally `STD_OUT`)
- `grep` to filter the content of the input (`STD_IN`) and print it on the output (`STD_OUT`) given a regexp
- `top` to pretty-print the content of the `/proc` folder and output some info on the running processes.

We also took some artistic liberty to implement some fun things to demonstrate what we can do.

#### Flex

- `neofetch`, to display some misc info on the OS (totally accurate)
- `snake` a little snake game, implemented in 36 hours to distract the unwary developer from implementing even more syscalls.
- `music` a little music player.

# Drivers

In order to be able to interact with various hardware elements, we wrote a couple basic drivers.

All of them implement the `Partition` Trait, to be able to fit into the FVS :

```rust
pub trait Partition {
    fn open(&mut self, path: &Path, flags: OpenFlags) -> Option<usize>;
    fn read(&mut self, oft: &OpenFileTable, size: usize) -> Result<Vec<u8>, IoError>;
    fn write(&mut self, oft: &OpenFileTable, buffer: &[u8]) -> isize;
    fn flush(&self);
    fn lseek(&self);
    fn read_raw(&self);
    fn close(&mut self, oft: &OpenFileTable) -> bool;
    fn give_param(&mut self, oft: &OpenFileTable, param: usize) -> usize;
}
```

This trait gives each driver a unified interface and enables the user to interact with each driver through a unique set of syscalls and through the VFS.

## VGA

The most important driver in our system is obviously the screen! We opted for a standard text-based interface, as we didn't want to mess with video-mode and pixel-fonts (we made a few tests of video-mode and concluded we would not need the capabilities of a video-based display).

For the display, we have multiple stacked interfaces. The most low-level one, in [`vga/mainscreen.rs`](../../src/vga/mainscreen.rs), handles all the basic logic associated  with a text-based terminal : it contains a reference to a fixed-sized buffer located at `0xb8000` and writes the individual characters into that buffer.

As we wanted our system to be "multitasking-ready™", we needed some form of multi-screen abstraction layer. That is what [`vga/virtual_screen.rs`](../../src/vga/virtual_screen.rs) is for. The basic idea is the following : each process is given a `VirtualScreen`, instantiated with a null size. It is free (obviously, in a real system this would require some sort of permissions) to change its size ans location on the `MainScreen`. It can write into that `VirtualScreen` transparently, as if it was a free-sized physical screen (this is done through the `VFS` and take into account some special characters to change color, cursor location, etc.), even though a `VirtualScrene` is simply an abstract structure containing a buffer. Each `VirtualScreen` is associated a `Layer`, which determines whether it must be displayed on top of other screens it may collide with.

The `MainScreen` (instantiated at startup) contains all `VirtualScreen`s in a priority-queue, indexed by layer. Whenever a `VirtualScreen` is written into, the `MainScreen` gets updated. It simply loops through all `VirtualScreen`s by order of increasing `Layer` and copies them into its buffer, at the given location and size.

This way, we can have multiple process display text onto the screen concurrently, sharing it and/or overlapping one another while preserving a unified interface for the screen : from within a process, we do not need to worry about the offset of the `VirtualScreen`, we simply write into it as if we could use the entire screen! 

There is however a small catch : as mentioned, the size and location (location on the physical screen) of a process' `VirtualScreen` can be changed by that process, which could potentially cause some issues. If we had more time, we would have implemented some sort of basic window management system, like a simplified `i3`. Being aware of this limitation, we are still pretty proud of that part of the project, as it seems to us really nice being able to edit some text while a clock is asynchronously displaying the time on the top part of the screen!

## Keyboard and mouse

The other important driver we wrote is the keyboard one! There is nothing really special : we activated the corresponding interrupt and got some structures setup containing a `VecDeque` to store the incoming scan-codes from the keyboard. Whenever a program reads bytes from `/hardware/keyboard`, these scan-codes are popped from the queue and handed over to the process. We also implemented the same structure for the mouse, which can be accessed through `/hardware/mouse`, even though we currently have no user-program using the mouse!

## Sound

As we wanted to be able to offer the user a complete experience, we needed some sort of basic sound driver. We therefore use the integrated beeper as an 70s-style single-voice speaker. To be able to take full advantage of that horrible-sounding beeper, we wrote a "complex" interface build around a priority-queue. Each sound event is encoded into a `SoundElement`, containing some information, like its tone, length and time-offset (from current time) it should start on. This way, we can provide the driver with a series of noted each with an offset, in order to play some music without worrying about the program accessing the speaker are the right time for the next note. 

The priority-queue uses as priority a tuple `(time_to_start, sound_id)` so that the sound popped from the queue is always the one that the driver needs to start playing. This also handles the situation of having multiple sound overlap (e.g. sound A starts at `1` and lasts for `2` ticks and B starts at `2` and lasts for `2` ticks : A plays from `1` to `3` and B from `3` to `4`.)

We simply keep track of the time by having a simple dummy variable incremented at each timer-interrupt (because the sound-driver updates its state at each such interrupt).

This driver provides a simple interface : [sound/mod.rs](../../src/sound/mod.rs).

There are some things we could have done if we had more time :

* add a `repeat` field into `SoundElement` so that one can get a repeating sound.
* move from a beeper-based driver to a more advanced one using buffers, etc. This way, we could have provided a mroe complex interface using a number of different voices.

## Clock

We have a straight-forward (using OSdev) driver that reads the CMOS RTC and outputs the time informations. It can be accesses by the kernel or a program through `/hardware/clock`.

## UsTar / Disk interaction

We really wanted to have a way to load and store data in a persistent way, using Qemu's disk emulation. Therefore we had to choose a file system format, so we took whatever the most simple one was : Tar. It went through some modifications and simplifications as we didn't need all features the original Tar format offers.

First, we wrote a simple ATA-disk driver. It required extensive testing and debugging because of some mistakes in integers casts that lead to incorrect addresses to be read/written to. Once we got this driver working, we moved onto the definition of our file format. The disk is divided in 512 bytes long sectors. Each group of 512 sectors form an LBA, sort of a meta-sector : in each LBA, the first sector hold an integer : index of the first sector in that LBA that is available as well as a table of the remaining 510 (and not 511 because the index takes up 2 bytes. So, basically, the 511-th sector of each LBA is wasted in the sake of simplicity of implementation) which indicates whether each sector is free. When the driver initializes, it reads the disk's length, deduce from it the number of LBA and then parses all LBA-allocation tables into memory.

Then, a super-allocation table is created, indicating whether each LBA has an available sector. Thanks to these tables, the allocation of sectors can be pretty quick because it is not needed to scan the whole disk to find available sectors.

On disk, a file (be it strictly speaking a file of a directory. We do not support other types of data e.g. symlinks) is composed of a header and an arbitrary number of data blocks. Both the header and data blocks take up exactly one sector. The header contains various informations : id of the creator, some flags, the file's name, its parent's address and, most importantly, the addresses of its data blocks (a header can contain up to 100 of them). This means that a file's blocks (be it header or data) do not need to be contiguous. This is in our humble opinion, a very important property of our file system.

But one could argue : a file can only have a size of $100 \times 512$ bytes. This is pretty small. To tackle this problem, we introduced different modes. If a file is small enough, it gets stored in `SMALL_MODE`, as described. In the other case, in gets stored in `LONG_MODE`. The address table in the header gets daisy-chained : the addresses (up to 100) in the header point to blocks that contains the addresses of the actual data blocks! That way, a file can have a size of up to $100 \times 128 \times 512 ~= 6.4$ MB. As Rust programs compiled with the `release` flag tend to be pretty small, we decided this was enough (the kernel itself is only a few megabytes long).

This mode, as well as the file type, is stored inside the header. A directory is stored in a pretty straightforward way : its data blocks contain tuples `(name, address)` of their children.

Because our OS isn't self-hosted, all user-programs were written and compiled on our machines. In order to insert them into the Tar filesystem, we wrote a PYthon script that scrapes a `filesystem` directory (included in our repo), containing the directories and files of the Tar filesystem, and turns it into a disk image following oure Tar format. It has been quite a pain but we finally got it to work! It is super convenient.

# Reliability

This is sort of the elephant in the room. From the very beginning, we built FerrOS to be a proof of concept of a feature-rich OS that, the code of which remains understandable. The goal was not for it to be reliable in any way. This means that it should work fine as long as the user follows the guidelines but will panic (i.e. crash on purpose) on multiple occasions. The UsTar driver is a great example of that : each step of the UsTar pipeline has a lot of data checks that cause the OS to crash if something invalid gets detected. A real OS would obviously catch this error and returns it to the user program, possibly killing it.

# Conclusion

This project has been a lot of fun and taught us a lot about operating systems, ranging from paging to user space and standard libraries. Using Rust for this project has been, in our humble opinion, a success, as it enabled us to build some complex structures relatively easily (such as the VFS).
