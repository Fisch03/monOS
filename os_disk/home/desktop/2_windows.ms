// one unique concept to monoscript are 'hooks'.
// these are your way to interact with the operating system running monoscript.
// let's take a closer look!

// a hook consists of its name, followed by a block of code surrounded by curly braces.
// the 'window' hook spawns new windows on the screen once it is executed.
window {
  // whenever the operating system wants to draw the contents of the window, it will execute the block of code inside the hook.
  // you can use various drawing functions to add content to the window:
  
  box(15, 15, 30, 60)   // rectangles using the 'box' function. it takes four arguments: x coordinate, y coordinate, width, height.
  color(0, 255, 0)      // the 'color' function sets the color for the shapes drawn after it. it takes three arguments: red, green, blue, each in the range 0-255.
  square(100, 75, 100)  // squares using the 'square' function. it takes three arguments: x coordinate, y coordinate, size.
}

// of course, you can have multiple hooks in your script. lets add another window!
// this time, lets also give the window a title. if you add a string in brackets after the hook name, it will be used as the window title!
window("second window!") {
    square(100, 200, 100)
}

// be careful though. since the code inside a window hook runs with each frame, you should avoid something like this:
// window {
//   // this runs whenever the outer window is drawn...
//   window { // ...creating a new window every frame!
//   }
// }
// this code would create up to 60 windows per second, which is probably not what you want :P 
// (there is a hard limit on 16 windows for your own safety though, so feel free to experiment around)
// the general rule is to keep the content of hooks as minimal as possible, especially avoiding nested hooks.

