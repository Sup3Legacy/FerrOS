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

::: incremental
- User space séparé du noyau, ELF-loader
- Ordonnancement préemptif par loterie
- Multi-processus, multi-screen
- VFS universel $\rightarrow$ accès à toutes les ressources depuis le userspace via une interface unifiée
- Gestion de processus depuis le userspace : `fork`, `exec`, `dup`
- Logiciels multimédia
:::

# Choix du langage

> But why would you want to use Rust instead of the all-mighty C? Rust's "safeness" comes with a great amount of limitations and those who give up their liberty for the sake of winning some temporary safety get neith#(IAç]/l5Q¦Bmçtl¿(Fx **Segmentation fault (core dumped)**

- Accès à des fonctionnalités tout aussi bas niveau (avec la version nightly et les macros `asm!`).
- Performances comparables au C (grâce à LLVM).
- Pas besoin de cross-compiler, il suffit de préciser les méta-données sur la cible de compilation.

-------

::: incremental
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
:::

---

# Macros for the keyboard layout
From 1000 to 250 lines.

```rust
macro_rules! layout {
    ( ; $( $k:literal $c:literal ),* ; $( $special:tt )* ) => {
        {
            let mut l: [Effect; 128] = [Effect::Nothing; 128];
            layout!(l ; $( $k $c ),* ; $( $special )*)
        }
    };
    ( $l:expr ; $( $k:literal $c:literal ),* ; $( $special:tt )* ) => {
        {
            $(
                {
                    $l[$k] = Effect::Value(KeyEvent::Character($c));
                }
            )*
            layout!($l ; $( $special )*)
        }
    };
    ( $l:expr ; $( $k:literal $sk:literal ),* ) => {
        {
            $(
                {
                    $l[$k] = Effect::Value(KeyEvent::SpecialKey($sk));
                }
            )*
            $l
        }
    };
}
```

---

# Parser combinators
```rust
/// Alternative parser combinator. Tries the rightmost parser first.
macro_rules! alt {
    ($s: expr ; $p: ident) => {
        $p($s)
    };

    ($s: expr ; $p: ident | $( $tail: ident )|* ) => {
        (alt! { $s ; $( $tail )|* }).or($p($s))
    };
}

/// Parser combinator: uses the parsers from left to right.
macro_rules! then {
    ($s: expr ; $l: expr) => {
        $l($s)
    };

    ($s: expr ; $l: expr => $( $tail_p: expr)=>+ ) => {
        $l($s).and_then(|(tail, _)| then! { tail ; $( $tail_p )=>+ })
    };
}
```

---

# Rust has good error messages except this one...

![Closures + Lifetimes parameters + Macros + Nightly = WTF!?](images/error.png)

---

# Other features
- Automatic documentation building
- Reproducible builds with `Cargo.lock`
- Conditional compilation: auto tests

---


# Partie technique

## x86-64

Moins d'inline que ce qu'on aurais du avoir autrement grâce aux abstractions de la librairie x86-64 de rust:

- `Cr2::read(); Cr3::write(...)`
- `Port::new()`

Très peu d'assembleur dans notre code. Seulement:
- Interruptions
- Kernel space $\rightarrow$ user space

## Mémoire virtuelle

Tout est en mémoire virtuelle en mode 64bits

- Bitmap Allocator

## bootloader

- Repris un bootloader écrit en Rust + ASM
- Tentative ratée
- Ajout d'une surprise dans le code du bootloader

## Syscall

- Vieux jeu $\rightarrow$ via les interruptions
- `80h` comme Linux
- Arguments : conventions d'appel du C


# Scheduler

- Ordonnanceur préemptif stochastique, gérant les priorités (à base de loterie).

| Ticket      | 7        | 6        | 5        | 4        | 3        | 2        | 1        | 0        |
| ----------- | -------- | -------- | -------- | -------- | -------- | -------- | -------- | -------- |
| Probabilité | $2^{-7}$ | $2^{-7}$ | $2^{-6}$ | $2^{-5}$ | $2^{-4}$ | $2^{-3}$ | $2^{-2}$ | $2^{-1}$ |

- Le ticket choisi donne la priorité minimale à exécuter (s'il n'y en a pas, on choisit parmi les priorités supérieures).
- Parmi les processus à priorité égale, on fait une bobine simple (round-robin).

---

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

```rust
pub trait Partition {
  fn open(&mut self,
    path: &Path,
    flags: OpenFlags) -> 
    Option<usize>;
  fn read(&mut self,
    oft: &OpenFileTable,
    size: usize) -> 
    Result<Vec<u8>, IoError>;
  fn write(&mut self,
    oft: &OpenFileTable,
    buffer: &[u8]) -> 
    isize;
```

----

```rust
  fn flush(&self);
  fn lseek(&self);
  fn read_raw(&self);
  fn close(&mut self,
    oft: &OpenFileTable) -> bool;
  fn give_param(&mut self,
    oft: &OpenFileTable, 
    param: usize) -> usize;
}
```

# Conclusion

https://wiki.osdev.org/Creating_an_Operating_System

- Étape 1:
  Tout sauf :
  - Internal Kernel Debugger 
  - Multithreaded Kernel 

---

- Stage 2: User-space
- [x] User-space
- [x] Lancement de programmes
- [x] System calls
- [x] OS Specific Toolchain 
- [x] Créer une bibliothèque ~~C~~ *Rust* 
- [x] Fork et Execute 
- [x] Shell

--- 

- Stage 3: Extending your Operating System 
- [x] Temps
- [ ] Threads
- [ ] Symmetric Multiprocessing 
- [x] Stockage secondaire  
- [x] Vrai système de fichier
- [ ] Interface graphique
- [ ] Interface utilisateur
- [ ] Mise en réseau
- [x] Son
- [ ] Universal Serial Bus


---

# Demonstration
auto tests
colors
multiscreen
clock (multiscreen)
userspace
- shell (|, >, >>, <, <<, &)
- cat (VFS)
- hexdump
- echo
- grep
- top
- neofetch
- snake
- music

---
