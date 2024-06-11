<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_peek.png" />

# the what
monOS is a 64-bit monolithic hobby OS written in Rust, inspired by the vtuber [Mono Monet](https://www.youtube.com/@MonoMonet) of V4Mirai.

### monoscript
while monOS may not be written in its own programming language, it comes with one! 
monoscript is monOS's very own scripting language, designed to be as simple as possible. you can use it to quickly write tools and even small games for monOS.
writing a "game" where you move a box around is as simple as:
```js
// define two variables for the box position
x_pos = 0
y_pos = 0

// create a window...
window {
  // ...and draw a red square at the x and y positions, with width/height 30
  square(x_pos, y_pos, 30, RED)
}

// change the box position while W/A/S/D is pressed
key(w) { y_pos -= 1 }
key(s) { y_pos += 1 }
key(a) { x_pos -= 1 }
key(d) { x_pos += 1 }
```
you can find more info about monoscript [here](https://github.com/Fisch03/monOS/tree/master/monoscript). you can also view it's documentation from inside monOS (soon™).

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_hmm.png" />

# the why
making a hobby OS has been something i've wanted to do for quite a while, but never quite got around to.
when i watched mono's [TempleOS stream](https://www.youtube.com/watch?v=xhbR5h6lw98), it sort of brought the idea back into my head - together with a quite obvious theme :P

for the same reason, monOS has quite a lot of references to TempleOS.

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_nerd.png" />

# the boring stuff (aka implementation details)
because i like torturing myself (and for educational reasons) the end goal for this kernel is to use no external dependencies apart from the [bootloader](https://github.com/rust-osdev/bootloader).
i handle this by first getting a basic implementation running using existing crates and then gradually substituting it with my own implementations.
this makes things a lot less painful than writing everything from scratch directly.

~~monOS is bootable from both BIOS and UEFI. that being said, most of my own testing was done on UEFI. therefore i recomend always using the UEFI image if you can!~~
BIOS boot is currently utterly broken and will not be supported going forward, sorry. maybe sometime later...

### project structure
- [`monos_kernel`](https://github.com/Fisch03/monOS/tree/master/monos_kernel) contains the kernel code (aka the main thing)
- [`monos_std`](https://github.com/Fisch03/monOS/tree/master/monos_std) is the library that userspace programs link against.
- the [`userspace` directory](https://github.com/Fisch03/monOS/tree/master/userspace) contains the included userspace programs that get shipped together with the kernel. these currently include:
  - [`hello_world`](https://github.com/Fisch03/monOS/tree/master/userspace/hello_world) basic template application
  - [`rooftop`](https://github.com/Fisch03/monOS/tree/master/userspace/rooftop) monOS's desktop environment
- the [`os_disk` directory](https://github.com/Fisch03/monOS/tree/master/monoscript_emu) will be used to construct the ramdisk (see below)
- [`monoscript`](https://github.com/Fisch03/monOS/tree/master/monoscript) is a (platform independent) library containing the monoscript parser and runtime
- [`monoscript_emu`](https://github.com/Fisch03/monOS/tree/master/monoscript_emu) contains a minimal monoscript runner for windows/linux/macOS

### devices
only PS2 mouse/keyboard support for now. i want to implement USB at some point but it seems like a huge pain. most vms and even hardware emulates PS2 from USB devices anyways.
monOS does use the newer APIC instead of the old fashioned 8259 PIC since its support seems to be (basically?) nonexistent under UEFI.

### memory
the kernel is mapped into the higher half of memory (at `0xffff800000000000`) and currently has a stack size of 1MiB and a heap size of 16MiB.
i would have raised this further if it wasn't for the fact that the page allocator currently is slow as heck (mostly because it doesn't allow allocating bigger blocks at once).
allocating the kernel heap already makes up most of boot time as-is, so i'd rather not raise it until that's solved. it's not like monOS uses even remotely close to the full 16MiB anyways.

#### allocation
monOS currently uses the following algorithms for allocating memory:

| type of memory    | allocation algorithm  |
| ----------------- | --------------------- |
| pages             | bitmap allocator      | 
| heap memory       | linked list allocator |
| virtual addresses | bump allocator        |
| userspace heap    | linked list allocator |

these were basically all just chosen because they were the easiest to implement. memory performance really isn't a big concern for this OS. 
as for virtual addresses, i would like to switch over to something more robust at some point. since monOS is strictly 64-bit with the kernel mapped into the higher half,
there is around 256TiB of virtual memory available for allocation. i just dont see that being reached over the time a single boot with the current scope of the project.

the heap allocator is using the [linked_list_allocator](https://github.com/rust-osdev/linked-list-allocator) crate right now, because i couldn't be bothered to write my own.
implementing my own is something i still want to do at some point though.

#### filesystem
monOS (currently) has no way of accessing external disks, all data is kept within a ramdisk. this means that all data gets wiped when the os reboots. feel free to wreak havoc :P.
the ramdisk itself is a FAT16 image with the following structure:
- `/bin`: userspace programs
- `/home`: user directory
- `/data`: OS resources. you probably shouldn't touch these unless you want to break something >:D

the build script automatically builds all the crates in the [`userspace` directory](https://github.com/Fisch03/monOS/tree/master/userspace) and puts their elf binaries in `/bin` 

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
  - [x] virtual address allocation
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
  - [x] process spawning
  - [ ] scheduler
- [ ] getting to userspace
  - [x] map kernel to upper half
  - [x] it works!
  - [x] elf file loading
    - [ ] using own implementation
  - [ ] syscalls
    - [x] it works!
    - [ ] framebuffer access
    - [ ] spawning/killing processes
    - [ ] filesystem access
  - [ ] userspace memory
    - [x] heap
    - [ ] sensible memory structure
    - [ ] ondemand paging
    - [ ] free on process exit
  - [ ] ipc
  - [ ] get sse to work
  - [ ] running doom
    - [ ] figure out linking
    - [ ] scuffed libc port
    - [ ] it works!
- [ ] monoscript
  - [x] basic parsing
  - [ ] missing statements
    - [ ] if/else
    - [ ] while
    - [ ] sound hook
  - [x] running in the emulator
  - [ ] running in the real thing
  - [ ] docs
  - [ ] improve execution performance
- [ ] monodoc
  - [ ] parsing
  - [ ] viewing
  - [ ] editing
- [x] filesystem support
  - [x] ramdisk reading
  - [x] fat16 drivers
  - [ ] block device drivers
- [ ] multiprocessor support (maybe)
- [ ] USB support (maybe)
- [ ] whatever else comes to mind :)

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_cheers.png" />

# big thanks
- to mono for being cool and based and providing me with lots of entertainment while i wrote this thing :3
- to my friends for putting up with me constantly bothering them with this project

- to Philipp Oppermann's [amazing blog series](https://os.phil-opp.com/) for getting me on the right track
- to this [blog post](https://nfil.dev/kernel/rust/coding/rust-kernel-to-userspace-and-back/) for getting me on the right track with getting into userspace
- to all the developers of the [x86_64 crate](https://github.com/rust-osdev/x86_64) that served as a great reference point for my own implementations
- to [moros](https://github.com/vinc/moros), [Hermit OS](https://github.com/hermit-os) and [Redox OS](https://www.redox-os.org/) for also serving as references
- to the [OSDev Wiki](https://wiki.osdev.org)


<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_smile.png" />

# credits
please contact me if you want your work removed. i wasn't able to find decent licensing information for all of these and am using them under the assumption that using for non-profit fan content is fine.
- [woofycakes](https://x.com/woofycakes) for the mono emotes in this file
- [NOiiRE ❖](https://noiire.carrd.co/) for the [mono cursor](https://x.com/noiireism/status/1736755359308792252)
- [slavfox](https://github.com/slavfox) for making [Cozette](https://github.com/slavfox/Cozette) (the system font)
