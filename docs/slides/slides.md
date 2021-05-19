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

- Tous les drivers dans le kernelspace
- Drivers simplifiés

Features :

- Userspace séparé du noyau, ELF-loader
- Ordonnancement préemptif par lotterie
- Multiprocessus, multiscreen
- VFS universel -> accès à toutes les ressources depuis le userspace via une interface unifiée
- Gestion de processus depuis le userspace : fork, exec, dup


## Plan

- Choix du langage, pros/cons (Paul)
- Partie technique x86-64, bootloader, syscalls (Samuel)
- Scheduler (Paul)
- VFS, interfaces, librust (Constantin)
- User-programs (peut-être aussi Rust?) (Gabriel)

# Choix du langage

> But why would you want to use Rust instead of the all-mighty C? Rust's "safeness" comes with a great amount of limitations and those who give up their liberty for the sake of winning some temporary safety get neith#(IAç]/l5Q¦Bmçtl¿(Fx **Segmentation fault (core dumped)**

---
Enfin ça dépend 
- jsp quoi
- Sécurité et fiabilité des structures déja existantes
- Compilation et gestion des dépendances



# VFS

## Drivers

Plusieurs drivers (en kernelspace) :

- Écran
- Clavier, souris
- Horloge, buzzer
- RAM-disk, pipe
- Disque
- Données logicielles (`/proc/`)

Intérêt du VFS : Unifier tout cela

## VFS

Chaque driver est une structure implémentent le trait `Partition`: 

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

---

# Rust
Bug type lifetime
macro
    keyboard macro
    lisp combinators
doc
build reproductible
auto tests

---

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

---
