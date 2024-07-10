// sometimes you want to reuse pieces of code from multiple places
// this is where functions come in. lets make a simple function that creates a new window:
fn my_cool_function() { // declare a new function called `my_cool_window`
  // when the function is run print a string
  print("good mononing!")
}

// to actually run the code in this function, we can call it - as often as we want:
my_cool_function()
my_cool_function()

// sometimes you may want to input some values into your function:
fn my_cool_window(width) { // declare a function called `my_cool_window` that takes one input: `width`
  window {
    box(50, 20, width, 30) // draw a box with the width from the input
  }
}

// now you can call the function with different inputs:
my_cool_window(10)
my_cool_window(300)

// functions can even output ("return") values:
fn add(x, y) { // declare a function called `add` that takes two values as input: x and y
  return x + y // add the two values together and return/output the result
}

// you can now use the function just like you would use a variable in your code:
value = add(1, 2)
print(value)

print(add(5, 5))