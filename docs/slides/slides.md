---
title: FerrOS
author: Gabriel \textsc{Doriath Döhler}, Paul \textsc{Fournier}, Constantin \textsc{Gierczak-Galle}, Samuel \textsc{Vivien}
abstract: FerrOS
advanced-maths: true
advanced-cs: true
theme: metropolis
---

# Introduction

## Description de haut niveau

Noyau de type micro-monolithique™ :

- Tous les drivers dans le kernel space
- Drivers simplifiés

Features :

- User space séparé du noyau, ELF-loader
- Ordonnancement préemptif par loterie
- Multi-processus, multi-screen
- VFS universel $\rightarrow$ accès à toutes les ressources depuis le userspace via une interface unifiée
- Gestion de processus depuis le userspace : `fork`, `exec`, `dup`
- Logiciels multimédia


## Plan

- Choix du langage, pros/cons (Paul)
- Partie technique x86-64, bootloader, syscalls (Samuel)
- Scheduler (Paul)
- VFS, interfaces, librust (Constantin)
- User-programs (peut-être aussi Rust?) (Gabriel)

# Choix du langage

> But why would you want to use Rust instead of the all-mighty C? Rust's "safeness" comes with a great amount of limitations and those who give up their liberty for the sake of winning some temporary safety get neith#(IAç]/l5Q¦Bmçtl¿(Fx **Segmentation fault (core dumped)**

- Accès à des fonctionnalités tout aussi bas niveau (avec la version nightly et les macros `asm!`).
- Performances comparables au C (grâce à LLVM).
- Pas besoin de cross-compiler, il suffit de préciser les méta-données sur la cible de compilation.

-------

- Suite d'outils `cargo` très riche
  - `check`, `build`, `test`, `run`,...
  - `fmt`
  - `clippy`
  - Possibilité d'auto-fix un grand nombre de petits problèmes
  - Compilation et gestion des dépendances
  - Un fichier de configuration pour les dominer tous : `Cargo.toml`
- Permet plus d'abstraction dans le code grâce aux traits
- Possibilité de choix de la représentation mémoire (`#[repr(C)]` par exemple)
- Sécurité et fiabilité des structures déjà existantes, même en `#![no-std]`.
- Les `SEGFAULT` sont très rares, il faut les parquer dans des `unsafe{...}`
- Frustration avec le borrow-checker (mais c'est pour votre bien, promis)



# Partie technique

## x86-64

Moins d'inline que ce qu'on aurais du avoir autrement grâce aux abstractions de la librairie x86-64 de rust:

- `Cr2::read(); Cr3::write(...)`
- `Port::new()`

Très peu d'assembleur dans notre code. Seulement:
- Interruptions
- Kernel space -> user space

## Mémoire virtuelle

Tout est en mémoire virtuelle en mode 64bits

- Bitmap Allocator

## bootloader

- Repris un bootloader écrit en rust + asm
- Tentative ratée
- Ajout d'une surprise dans le code du bootloader

## Syscall

- Vieux jeu -> via les interruptions
- `80h` comme linux
- Arguments = Conventions d'appel du C


# Scheduler

- Ordonnanceur préemptif stochastique, gérant les priorités (à base de loterie).

| Ticket      | 7        | 6        | 5        | 4        | 3        | 2        | 1        | 0        |
| ----------- | -------- | -------- | -------- | -------- | -------- | -------- | -------- | -------- |
| Probabilité | $2^{-7}$ | $2^{-7}$ | $2^{-6}$ | $2^{-5}$ | $2^{-4}$ | $2^{-3}$ | $2^{-2}$ | $2^{-1}$ |

- Le ticket choisi donne la priorité minimale à exécuter (s'il n'y en a pas, on choisit parmi les priorités supérieures).
- Parmi les processus à priorité égale, on fait une bobine simple (round-robin).



# VFS

## Drivers

Plusieurs drivers (en kernel space) :

- Écran
- Clavier, souris
- Horloge, buzzer
- RAM-disk, pipe
- Disque
- Données logicielles (`/proc/`)

Intérêt du VFS : Unifier tout cela

## VFS

Chaque driver est une structure implémentent le trait `Partition`: 



# Rust
Bug type lifetime
macro
    keyboard macro
    lisp combinators
doc
build reproductible
auto tests

# Demonstration
colors
userspace
- shell
- cat
- grep
- echo
- hexdump
launcher
neofetch
clock
multiscreen
VFS
musique

# Conclusion

https://wiki.osdev.org/Creating_an_Operating_System

- Stage 1: Beginning
Everything except
  -> Internal Kernel Debugger 
  -> Multithreaded Kernel 

- Stage 2: User-space
- [x] User-space
- [x] Program loading
- [x] System calls
- [x] OS Specific Toolchain 
- [x] Creating a -C- *Rust* Library
- [x] Fork and Execute 
- [x] Shell

- Stage 3: Extending your Operating System 
- [x] Time
- [ ] Threads
- [ ] Symmetric Multiprocessing 
- [x] Secondary Storage  
- [x] Real Filesystems
- [ ] Graphics
- [ ] ? User Interface
- [ ] Networking
- [x] Sound
- [ ] Universal Serial Bus
