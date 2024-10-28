<p align="center">
<img height="125" src="https://github.com/Fisch03/monOS/blob/master/img/logo.png" />
</p>
<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_peek.png" />

# the what
monOS is a 64-bit monolithic hobby OS written in Rust, inspired by the vtuber [Mono Monet](https://www.youtube.com/@MonoMonet) of V4Mirai (and TempleOS, sorta).

this project is still very much in active development. dont expect anything usable (or working) at all. i would consider this usable once most of the points on the [big todo list](#the-big-todo-list) are done. if you want to try out the os regardless, check [here](#buildingrunning) for some more info

below are some cool things monOS will ship with:

### monoscript (work in progress)
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
you can find the monoscript source [here](https://github.com/Fisch03/monOS/tree/master/monoscript). you can also view it's documentation from inside monOS (soon™). there is also the very basic [`monoscript_emu`](https://github.com/Fisch03/monOS/tree/master/monoscript_emu) crate that lets you run monoscript on windows/mac/linux

### monodoc (draft, implementation pending)
accompanying monoscript, there is monodoc. much like TempleOS's DolDoc (if you are familiar with TempleOS, this is basically what every part of the ui except the top bar and window management consists of). monodoc this is a file format for writing text files that can do more than just text!
it is a superset of markdown (and even shares its file extension - `.md`). this means its super simple to use, and you can read and edit monodoc files with basically all text editors and markdown viewers 
while only loosing out on some of the fancier features. these special features include:
- in-document monoscript execution and rendering - play animations or even entire games right inside monodoc documents
- special links - various actions inside monOS can be executed by clicking one of these links. for example a link with the location (mo:exec/hello_world) will launch the `hello_world` program when clicking it.

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_hmm.png" />

# the why
making a hobby OS has been something i've wanted to do for quite a while, but never quite got around to.
when i watched mono's [TempleOS stream](https://www.youtube.com/watch?v=xhbR5h6lw98), it sort of brought the idea back into my head - together with a quite obvious theme :P

for the same reason, monOS has(/will have) quite a lot of references to TempleOS.

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_nerd.png" />

# the boring stuff (aka implementation details)
because i like torturing myself (and for educational reasons) the end goal for the kernel is to use no external dependencies apart from the [bootloader](https://github.com/rust-osdev/bootloader).
i handle this by first getting a basic implementation running using existing crates and then gradually substituting it with my own implementations.
this makes things a lot less painful than writing everything from scratch directly.

~~monOS is bootable from both BIOS and UEFI. that being said, most of my own testing was done on UEFI. therefore i recomend always using the UEFI image if you can!~~
BIOS boot is currently utterly broken and will not be supported going forward, sorry. maybe sometime later...

### project structure
- [`monos_kernel`](https://github.com/Fisch03/monOS/tree/master/monos_kernel) contains the kernel code (aka the main thing)
- [`monos_std`](https://github.com/Fisch03/monOS/tree/master/monos_std) is the library that userspace programs link against.
- [`monos_gfx`](https://github.com/Fisch03/monOS/tree/master/monos_gfx) is the ui library used by graphical applications
- the [`userspace` directory](https://github.com/Fisch03/monOS/tree/master/userspace) contains the included userspace programs that get shipped together with the kernel. these currently include:
  - [`hello_world`](https://github.com/Fisch03/monOS/tree/master/userspace/hello_world) basic template application
  - [`rooftop`](https://github.com/Fisch03/monOS/tree/master/userspace/rooftop) monOS's desktop environment
- the [`os_disk` directory](https://github.com/Fisch03/monOS/tree/master/os_disk) will be used to construct the ramdisk (see below)
- [`monoscript`](https://github.com/Fisch03/monOS/tree/master/monoscript) is a (platform independent) library containing the monoscript parser and runtime
- [`monoscript_emu`](https://github.com/Fisch03/monOS/tree/master/monoscript_emu) contains a minimal monoscript runner for windows/linux/macOS

### devices
only PS2 mouse/keyboard support for now. i want to implement USB at some point but it seems like a huge pain. most vms and even hardware emulates PS2 from USB devices anyways.
monOS does use the newer APIC instead of the old fashioned 8259 PIC since its support seems to be (basically?) nonexistent under UEFI.

### memory
the kernel is mapped into the higher half of memory (at `0xffff800000000000`) and currently has a stack size of 1MiB and a heap size of 16MiB.

#### layout
| type               | start                  | size   | page table indices |
| ------------------ | ---------------------- | ------ | ------------------ |
| userspace code     |              0x200_000 |        | (  0, 0, 1, 0, 0)  |
| userspace heap     |       0x28_000_000_000 |        | (  5, 0, 0, 0, 0)  |
| userspace stack    |      0x400_000_000_000 | 4 MiB  | (128, 0, 0, 0, 0)  |
| mapped framebuffer |      0x410_000_000_000 |        | (130, 0, 0, 0, 0)  |
| ------------------ | ---------------------- | ------ | ------------------ |
| kernel code        | 0xffff_800_000_000_000 |        | (256, 0, 0, 0, 0)  |
| kernel heap        | after kernel           | 16 MiB |                    |

#### allocation
monOS currently uses the following algorithms for allocating memory:

| type of memory    | allocation algorithm  |
| ----------------- | --------------------- |
| pages             | bitmap allocator      | 
| heap memory       | buddy system          |
| userspace heap    | buddy system          |

these were basically all just chosen because they were the easiest to implement. memory performance really isn't a big concern for this OS. 

the heap allocator is using the [linked_list_allocator](https://github.com/rust-osdev/linked-list-allocator) crate right now, because i couldn't be bothered to write my own.
its something i still want to do at some point though.


#### filesystem
monOS (currently) has no way of accessing external disks, all data is kept within a ramdisk. this means that everything gets wiped when the os reboots. feel free to wreak havoc :P.
the ramdisk itself is a FAT16 image with the following structure:
- `/bin`: userspace programs
- `/home`: user directory
- `/data`: OS resources. you probably shouldn't touch these unless you want to break something >:D

the build script automatically builds all the crates in the [`userspace` directory](https://github.com/Fisch03/monOS/tree/master/userspace) and puts their elf binaries in `/bin`

monOS's vfs model closely follows the one linux uses (albeit much simplified). the vfs is represented as a tree structure that gets lazily loaded into memory as needed.
vfs nodes currently are either files or directories, i dont think ill add symlinks since they would complicate links a lot further.
one filesystem gets used as the root and then other filesystem implementations can be mounted inside empty directories (this is currently unused though...).

i have thought about replacing the current [messaging](#messaging) functionality with virtual files aswell since it would simplify the usage from userspace. 
the kernel implementation would be relatively complex though and also probably not as fast so itll stay as a idea for now.

### syscalls
the following syscalls currently exist: 
| id | name         | param 1                      | param 2        | param 3                         | param 4 | description                                                                            | 
| -- | ------------ | ---------------------------- | -------------- | ------------------------------- | ------- | -------------------------------------------------------------------------------------- |
| 0  | spawn        | file path ptr                | file path len  |                                 |         | spawn a new process of the given binary                                                |
| 1  | yield        |                              |                |                                 |         | suspend the current process and move it to the end of the process queue                |
| 2  | serve        | port name ptr                | port name len  |                                 |         | provide a channel on the given port                                                    |
| 3  | connect      | port name ptr                | port name len  | ptr to `Option<ChannelHandle>`  |         | connect to a channel at the given port                                                 |
| 4  | wait_conn    | port name ptr                | port name len  | ptr to `Option<ChannelHandle>`  |         | wait for a process to connect to a channel on the given port                           |
| 5* | send         | data 1                       | data 2         | data 3                          | data 4  | send data over a opened channel (asynchronously)                                       |
| 6* | receive      | ptr to `Option<Message>`     |                |                                 |         | block until data is received on a given opened channel                                 |
| 7  | receive_any  | ptr to `Option<Message>`     |                |                                 |         | block until data is received on any opened channel                                     |
| 8  | req_chunk    | requested size               |                |                                 |         | request a memory chunk of the given size. returns address of chunk or 0 on failure     |
| 9  | open         | file path ptr                | file path len  | ptr to `Option<FileHandle>`     |         | open a file at the given path                                                          |
| 10 | seek         | `FileHandle`                 | offset         |                                 |         | seek to a specific position in a opened file                                           |
| 11 | read         | `FileHandle`                 | buffer ptr     | buffer len                      |         | read len bytes from a opened file                                                      |
| 12 | write        | `FileHandle`                 | buffer ptr     | buffer len                      |         | write len bytes to a opened file                                                       |
| 13 | list         | file path ptr                | file path len  | ptr to `[ArrayPath; n]`         | arr len | list directory entries. reads only param 4 amt, you should stat the dir first          |
| 14 | print        | string ptr                   | string len     |                                 |         | print a string *somewhere* (serial port currently). should only be used for debugging. |

*the syscall id is a bit special for the `send`, and `receive` syscalls (see the chapter on messaging below).

#### messaging
inter-process communication in monOS happens over channels. a process can provide a channel on a port (basically just a unique string) using the `serve` sycall. 
other processes can then open a connection on the port using the `connect` syscall. this provides both the sending and the receiving process (using the `wait_conn` syscall) with a channel handle. 
both processes can then send and receive messages over the channel using the `send`, `receive` and `receive_any` syscalls.

a message is either scalar consisting of 4 64-bit values or a chunked message. a chunked message points to a previously requested (using the `req_chunk` syscall) memory chunk 
that will then be unmapped from the address space of the sending process and mapped into the address space of the receiving process.
some messaging related syscalls are a bit special since they use the syscall id to pass some additional parameters:
| bits  | content                             |
| ----- | ----------------------------------- |
|  0- 7 | syscall id                          |
|     8 | 0 = scalar values, 1 = memory chunk |
|  9-15 | nothing... for now :)               |
| 16-63 | channel handle                      |

a channel handle is a 48-bit value consisting of 32 bits target process id, 8 bit target channel id and 8 bit sender channel id.
there is currently no safety in place for channels, meaning that a process can just send to any channel knowing its channel handle without `connect`ing to it first. i should probably fix that at some point...

# the big todo list
- [x] it boots!
- [x] working framebuffer
  - [x] text drawing
  - [x] double buffering
- [ ] exception handling
  - [x] basic handlers 
  - [ ] better panic handling
    - [x] handle panics when the framebuffer isn't initialized yet
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
    - [ ] figure out why the allocator breaks when compiling with `-g`
    - [ ] implement own allocator
- [x] ACPI
  - [x] basic table parsing
  - [ ] (?)
- [ ] APIC
  - [ ] local apic
    - [x] timer interrupts
      - [ ] calibrate timer using PIT
  - [x] io apic
    - [x] ps2 keyboard input
    - [x] ps2 mouse input
- [ ] gui
  - [x] image loading
    - [x] .ppm loading
    - [x] turn fb into image for fast screen clearing
  - [ ] decently usable immediate mode gui library
    - [x] basic functionality
    - [x] better text wrapping
    - [x] buttons
    - [x] text input
    - [x] scrollable 
    - [ ] "drawing" functionality
    - [ ] only rerender if necessary
  - [ ] rewrite fb drawing code because it is messy as hell
  - [x] boot screen 
  - [ ] kernel panic screen
  - [ ] desktop environment (`rooftop`)
    - [x] basic ui
    - [x] only rerender necessary parts
    - [x] spawning windows
    - [x] usable client
    - [ ] keyboard input
    - [ ] moving windows
  - [ ] terminal
- [ ] task management
  - [ ] async executor
  - [x] process spawning
  - [x] scheduler
- [ ] getting to userspace
  - [x] map kernel to upper half
  - [x] it works!
  - [x] elf file loading
    - [ ] using own implementation
  - [ ] syscalls
    - [x] it works!
    - [x] framebuffer access
    - [ ] spawning/killing processes 
      - [x] spawning
      - [ ] killing
    - [x] filesystem access
  - [ ] userspace memory
    - [x] heap
    - [x] sensible memory structure
    - [x] ondemand paging
    - [ ] figure out why allocation on the heap fails at a certain point
    - [ ] memory chunks
    - [ ] free on process exit
  - [ ] ipc
    - [x] kernel <-> process
    - [x] sending memory chunks
    - [ ] block waiting processes
    - [x] process <-> process
    - [ ] mpsc/broadcasts (?)
  - [ ] save sse/avx context
  - [ ] running [REDACTED] :)
    - [ ] figure out linking
    - [ ] scuffed libc port
    - [ ] it works!
- [ ] monoscript
  - [x] basic parsing
  - [ ] missing statements
    - [x] function declarations 
    - [x] function return values
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
  - [ ] docs
- [x] filesystem support
  - [x] ramdisk reading
  - [x] fat16 drivers
    - [ ] multi-lfn file names
  - [x] decent vfs implementation
  - [ ] keep track of opened files to avoid conflicts
  - [ ] block device drivers
- [ ] networking
  - [ ] get access to the network card/pcie devices at all
  - [ ] get [smoltcp](https://github.com/smoltcp-rs/smoltcp) (or some other TCP stack) running
  - [ ] write a simple http implementation (or use an existing one, idk)
  - [ ] get websockets working
  - [ ] [Cibo Online!](https://github.com/Fisch03/cibo-online) client
- [ ] multiprocessor support (maybe)
- [ ] USB support (maybe)
- [ ] whatever else comes to mind (suggest something!) :)

<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_cheers.png" />

# building/running
you'll need a working rust installation. install llvm-tools (`rustup component add llvm-tools-preview`) and then do a `cargo build` in the workspace root. the disk image will be in `target/debug/build/monos-xxxxxxxxxxx/out/uefi.img`. if you have qemu installed, you can just do `cargo run` and it will be started automatically after the build :)

running in virtualbox seems to work great too. convert the image into a virtualbox `.vdi` disk image using qemu-img (`qemu-img convert -f raw -O vdi uefi.vdi uefi.iso`) or a similar tool. set the vm type to "Other" and the version to "Other/Unknown (64-bit)". make sure to tick the "Enable EFI (special OSes only)" option at the hardware step or it won't work.

# big thanks
- to mono for being cool and based and providing me with lots of entertainment while i wrote this thing :3
- to my friends for putting up with me constantly bothering them about this project <3
- to Philipp Oppermann's [amazing blog series](https://os.phil-opp.com/) for getting me started on this at all
- to this [blog post](https://nfil.dev/kernel/rust/coding/rust-kernel-to-userspace-and-back/) for getting me on the right track with getting into userspace
- to all the developers of the [x86_64 crate](https://github.com/rust-osdev/x86_64) that served as a great reference point for my own implementations
- to [EuraliOS](https://github.com/bendudson/EuraliOS), [moros](https://github.com/vinc/moros), [Hermit OS](https://github.com/hermit-os) and [Redox OS](https://www.redox-os.org/) for also serving as awesome references
- to the [OSDev Wiki](https://wiki.osdev.org)


<img width="64" align="right" src="https://github.com/Fisch03/monOS/blob/master/img/mono_smile.png" />

# credits
please contact me if you want your work removed. i wasn't able to find decent licensing information for all of these and am using them under the assumption that using for non-profit fan content is fine.
- [woofycakes](https://x.com/woofycakes) for the mono emotes in this file
- [slavfox](https://github.com/slavfox) for [Cozette](https://github.com/slavfox/Cozette), the system font
- [benwr](https://github.com/benwr) for [Glean](https://github.com/benwr/glean), the (smaller) system font and Jim Knoble for Neep, the font that Glean is originally based on
