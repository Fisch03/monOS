<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_peek.png" />

# the what
monOS is a 64-bit monolithic hobby OS written in Rust themed after the vtuber [Mono Monet](https://www.youtube.com/@MonoMonet) of V4Mirai.

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_hmm.png" />

# the why
making a hobby OS has been something i've wanted to do for quite a while, but never quite got around to.
when i watched mono's [TempleOS stream](https://www.youtube.com/watch?v=xhbR5h6lw98), it sort of brought the idea back into my head - together with a quite obvious theme :P

for the same reason, monOS has quite a lot of references to TempleOS.

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_nerd.png" />

# the boring stuff (aka implementation details)
because i like torturing myself (and for educational reasons) the kernel uses no external dependencies apart from the [bootloader](https://github.com/rust-osdev/bootloader).
during development i handled this by first getting a basic implementation running using the [x86_64 crate](https://github.com/rust-osdev/x86_64) and then gradually substituting it with my own implementations.
this made things a lot less painful than writing everything from scratch directly.

monOS is bootable from both BIOS and UEFI. that being said, most of my own testing was done on UEFI. therefore i recomend always using the UEFI image if you can!

### memory
frames are allocated using a basic bitmap allocator. heap memory is allocated using a linked list allocator.

#### kernel virtual address space (horribleness ahead)
i ~~plan to~~ really really need to automatically assign new virtual adresses at some point but for now this will have to suffice

| adress         | size  | mapped to               |
| -------------- | ----- | ----------------------- |
| 0xfee00000     | 4KiB  | local apic              |
| 0xfee10000     | 4KiB+ | acpi tables             |
| 0xfee30000     | 4KiB  | io apic                 |
| 0x123456780000 | ????  | framebuffer back buffer |
| 0x444444440000 | 1MiB  | kernel heap             |

# the big todo list
- [x] it boots!
- [x] working framebuffer
  - [x] text drawing
  - [x] double buffering
  - [ ] shapes
- [ ] exception handling
  - [x] basic handlers 
  - [ ] better panic handling
    - [ ] handle panics when the framebuffer isn't initialized yet
    - [ ] show a extra screen on panic
- [x] GDT/TSS for seperate exception stacks
- [ ] memory
  - [x] paging 
    - [x] address translation
    - [x] page mapping/unmapping
    - [x] frame allocation
      - [ ] use a better allocator (maybe)
  - [ ] heap allocation
    - [x] basic implementation
    - [ ] implement own allocator
  - [ ] virtual address allocation
- [ ] ACPI
  - [x] basic table parsing
  - [ ] (?)
- [ ] APIC
  - [ ] local apic
    - [x] timer interrupts
      - [ ] precisely timed timer interrupts
  - [x] io apic
    - [x] ps2 keyboard input
    - [x] ps2 mouse input
- [ ] gui
  - [ ] decently usable ui framework
    - [ ] mouse cursor
    - [ ] windows with seperate framebuffers
    - [ ] invalidation rectangles (or some other mechanic for only redrawing the screen if needed)
  - [ ] boot screen 
  - [ ] terminal
  - [ ] home menu
- [ ] task management
  - [ ] async executor
  - [ ] process spawning
  - [ ] elf file loading (maybe)
- [ ] MonoScript (name pending) 
- [ ] multiprocessor support (maybe)
- [ ] USB support (maybe)
- [ ] whatever else comes to mind :)

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_cheers.png" />

# big thanks
- to Philipp Oppermann's [amazing blog series](https://os.phil-opp.com/) for getting me on the right track
- to all the developers of the [x86_64 crate](https://github.com/rust-osdev/x86_64) that served as a great reference point for my own implementations
- to [moros](https://github.com/vinc/moros), [Hermit OS](https://github.com/hermit-os) and [Redox OS](https://www.redox-os.org/) for also serving as references
- to the [OSDev Wiki](https://wiki.osdev.org)
- to mono for being cool and providing me with entertainment while i wrote this thing
- to my friends for putting up with me constantly bothering them with this project

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_smile.png" />

# credits
please please please contact me if you want your work removed.
- [woofycakes](https://x.com/woofycakes) for the mono emotes
- [NOiiRE ❖](https://noiire.carrd.co/) for the [mono cursor](https://x.com/noiireism/status/1736755359308792252)
- [slavfox](https://github.com/slavfox) for making [Cozette](https://github.com/slavfox/Cozette) (the system font)
