# you made it!
if you see this file, you've probably just booted up monOS, welcome!

this file is supposed to give you some pointers about using monOS. 
feel free to explore around yourself though. you can always reopen this file from the file explorer (unless you delete it :P)

## navigation
TODO

## monodoc
monodoc is a simple markup language for formatting text files (that definitely has nothing to do with markdown). 
files with the ending `.md` will automatically be opened with monOS's built-in monodoc viewer. 
in fact - the file you are viewing right now is written in monodoc :D. 
you can press the [edit](md:edit) button in the upper right corner to see the actual contents of this file and play around with adding some of your own stuff!

monodoc can do all the things markdown does, and a lot more.
here is some basic syntax to get you started:

```
#  heading
## small heading
```

```
- creating
- a
- list
```

now for the really cool stuff. you can embed [monoscript](#monoscript) code right inside monodoc files! as long as the monoscript code creates a window, its contents will be rendered directly within the file
```!
window {
  // TODO
  box(20, 20, 10, 10)
}
```

## monoscript
TODO






