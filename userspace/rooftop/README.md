# rooftop
is monOS's desktop environment and also provides a library for both window servers and clients

## using the desktop environment
general usage should be more or less self explanatory. 
free to check out the monOS user guides on the desktop for more info on navigating and using monOS :)

### adding desktop entries
desktop entries are loaded from the `home/desktop` directory. 
they follow a very simple file format where each line has the format `key=value`. the following keys are available (unless noted these are all mandatory):
- `name`: determines the name of the desktop entry
- `icon`: path to a 32x32px `.ppm` file providing a icon for the desktop entry
- `open`: path to the binary that will be executed upon clicking the entry
- `args`: (optional) space seperated arguments to pass to the binary

## using as a library
### client
make sure to enable the `client` crate feature! `rooftop` is closely tied to [`monos_gfx`](https://github.com/Fisch03/monOS/tree/master/monos_gfx) so you should also add that as a dependency.

create a window client by specifying the port to connect to and some form of data containing your applications state. the port provided by monOS's desktop environment is `desktop.windows`.
```rust
use rooftop::WindowClient;

struct AppState {
  // ...
}

let mut client = WindowClient::new("desktop.windows", AppState { /* ... */ });
```

you can now create windows by specifying their title, initial size and a function that should be called on render:
```rust
let window = client.create_window("my app", monos_gfx::Dimension::new(320, 240), render);

fn render(state: &mut AppState, window: &mut Window, mut input: monos_gfx::Input) {
  // do your rendering here! the window implements Deref<Target=Framebuffer> so you can easily draw onto it!
}
```
finally, let your application run by entering a loop that repeatedly calls `client.update();`
```rust
loop {
  
  // you can also access the state here to perform rendering independent logic if needed:
  let state = client.data_mut();
  // ...

  client.update(); 

  //optional but highly recommended (see section on implementation details for more info)
  syscall::yield_(); 
}
```

#### update frequency
by default, the render function of your windows only gets called when the window receives any mouse/keyboard input. this works great for most simple gui applications, but not for more dynamic stuff. theres two ways to affect this:

##### changing the update frequency
theres three `UpdateFrequencies` available:
- `Always`: will always rerender the window as soon as the desktop is ready to render its next frame
- `OnInput`: the default - will rerender the window as soon as it has received new input
- `Manual`: will never rerender the window unless you manually request it (see below). note that this will lead to dropped inputs and should therefore only be done if youre fine with that.
theres two ways to change the `UpdateFrequency` of windows depending on where you are:
  - in your windows render function, you can directly set the update frequency: `*window.update_frequency = new_freq`
  - in your main loop/setup code, you can use the `WindowHandle` returned by the `client.create_window` function to change it: `client.set_update_frequency(handle, new_freq)`

##### manually requesting renders
in your main loop, you can use the `WindowHandle` returned by the `client.create_window` function to manually request updates at any point: `client.request_render(handle)`. this can be useful both for windows with a `Manual` update frequency, but also if you have a window with a frequency set to `OnInput` and detect some kind of other state change that requires a rerender

### server
you can also use rooftop as a library to provide a window server!
i wont document this too heavily since i dont expect anyone to actually do this but i recommend checking out rooftops main loop since it should show the functionality quite well.

## implementation details
the whole architecture of rooftop is built around the idea of immediate mode rendering and using as little redundancy as possible. upon request, the window server creates a chunk of data for each window containing a framebuffer and some additional data including id, dimensions and input.
this chunk is then also mapped into the window clients address space. from that point onwards, most communication happens via that shared memory chunk.

