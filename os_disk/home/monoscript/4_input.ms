// lets take a look at another type of hook: the 'key' hook

// first we do a bit of setup...
// define two variables for keeping track of position
x_pos = 0
y_pos = 0


// create a window...
window {
  // ...and draw a square at the x and y positions, with size 30
  square(x_pos, y_pos, 30)
}

// now the fun part: we can use the 'key' hook to respond to key presses!
// differently to the window hook, you also need to specify the key you want to respond to (e.g. 'key("w")')
// lets allow the user to move the box around using WASD.

// we'll also use a special new operator that makes our code a bit more readable: '+='
// instead of writing 'x_pos = x_pos + 3' to increase the x position by 3, we can simply write 'x_pos += 3'!
// this also works with other operators, such as '-=', '*=' and '/='
key("w") { y_pos -= 3 }
key("s") { y_pos += 3 }
key("a") { x_pos -= 3 }
key("d") { x_pos += 3 }

