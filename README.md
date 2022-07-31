# Getting started
## Installation
Installing the *CryScript Interpreter* is as simple as a click!
- For Windows, [click here](https://github.com/rookieCookies/CryScript/tree/master/builds/windows)
- For MacOS, [click here](https://github.com/rookieCookies/CryScript/tree/master/builds/macos)
- For Linux,
    1. Download the source code from the [github page](https://github.com/rookieCookies/CryScript)
    2. Download the Rust Toolchain from the [official page](https://www.rust-lang.org/tools/install)
    3. Extract the source into a folder
    4. Run `cargo build --release` and then wait for it to compile
    5. Navigate to the target folder and you should see another folder called release, after opening the release folder you should see the application

## Hello World!
Tryin' a new language is no easy task, fortunately CryScript makes it quite easy to start!
First, let's create a file that we will be entering our code into! For the purposes of this tutorial we'll call it `script.cry`
Now enter the following text into the file we created

>println("Hello world!")

That should be pretty self-explanatory.
We're calling a function called `println` and giving it the text "Hello World!"
and now all that's left is running the file!
Move the file you downloaded from the [installation section](##installation) and then run it!
If everything went as expected you'll be prompted with a text asking for a file path, simply enter `script.cry` and press enter

Voila! Hello World!  
# Common Concepts
## Variables
Variables are containers for storing different data values  
In CryScript there are a few types of variables  
* `Strings   `|  stores a text such as "Hello World!"  
* `Integers  `|  stores a whole number such as 987 or -123
* `Float     `|  stores a decimal number such as 0.324 or -5406.1
* `Function  `|  stores a function reference to be called later

### Declaring (Creating) variables
#### Syntax: `var variable_name = value`  
Let's look at that word by word,
* "var"  is just a keyword indicating that this is a variable declaration
* "variable_name" is the name of the variable such as (x or name)
* the equal sign is used to assign a value to the variable
* and then the value is the actual value we want the variable to be

Don't worry about trying to memorize all of these since you'll get used to it as we go on with examples, let's look at some examples shall we?   
   
#### Examples

* Declare a variable called "message" and then assign it to "Hi!"  
    >var message = "Hi!"
* Declare a variable called "message" and then assign it to 12  
    >var message = 12
* You can also declare a variable and assign it later  
    >var message  
    >message = "Yep!"
* Or even override an existing value
    >var message = 3  
    >message = 1 // Message is now 1 and the old value is overriden

### Final Variables
If you don't want a variable to be updated you can declare it using the final keyword  
This will make it so if the variable is updated later, it will throw an error
>final var example = "Hello"  
>example = "No" // This will throw an error since example is a final variable

### Typed Variables
#### Syntax: `var variable_name : type = value`  
In CryScript by default all variables are dynamic, which means they can be assigned to any type at any point, using typed variables will make it so if the variable is assigned to a different type it will throw an error  

Here are some examples!
* Defining a typed variable
    >var example_str : str = "Example Text"  
    >var example_int : int = 5  
    >var example_float : float = 12.5  
* Updating a variable with the wrong type will throw an error
    >example_str = 5 // Since 5 is an integer, it will throw an error
* But declaring variables with the wrong type will also throw an error
    >var example_func : fn = 32 // Error  

## Functions
..are a way to reduce repetitive code  
You have already used a function, at the [very start](#hello-world) you used the `println` function!
### Declaring Functions
Let's look at how to declare functions
>fn hello() {  
>    println("Hello World")  
>}  

Hmm, but how do you make it so you can pass it data? Just like `println`! Simple, we just add a parameter!
>fn hello(data) {  
>    println(data)  
>}  

Parameters in function declarations are variables that will be provided by the caller, they can even have types like normal variables!
>fn hello(data : str) {  
>    println(data)  
>}  

## Comments
Every programmer should strive to make their code as self-explanatory as possible, but sometimes this is just not viable, that's where comments come in!
The program will ignore these comments but it might be useful to other people reading your code!

> // hello world

Two slashes are used for single-line comments, these comments will continue until the end of the line

>/*  
>This is a multiple line comment  
> Woah the code under this must be really complicated!  
> I hope this comment is enough to cover everything!
>*/

The `/*` is used to start a multi-line comment, and it will continue until `*/` is found

## Control Flow
### If expressions
An `if` expression allows you to run code based on conditions, you can think of it was "if this is true run this"  
>var x = 5  
>  
>if x > 3 {  
>    println("x is bigger than 3")  
>} else {  
>    println("x is not bigger than 3)  
>}  

Even though line breaks are important in CryScript, one line statements like this are still possible
>var x = if 5 > 3 { true } else { false }  

### While expression
A `while` expression can be thought of as "run this piece of code until x is not true"
>var x = 10  
>  
>while x > 0 {  
>    x -= 1  
>}

# Let's make a guessing game!
Practice what we've learned so far!
## Generating a secret number
To make a guessing game we need a secret random number, luckily the standard library provides us with some random functionality. Though this feature needs to be brought in to be used  
This model is used to prevent unnecessary speed drops  

To bring the standard random library to use add the following line
>use "std_rand"

Great! Now we can generate a random number using the `rand_range_int` function  

>var secret_number = rand_range_int(0, 100)

We generate a random integer between 0 and 100 and assign it to the secret_number variable

Okay, we have a secret number but now we need a way to get the users input. Again, the standard library comes into the rescue. Using the `read` function we can get input from the console  
>var guess = read() as int

Now, there's something in that line we've not discussed.  
Since read() returns a string we need to parse it as a number so we can compare it with our secret number later on. To do that we use `as int`.  
Note that this statement can cause an exception to be thrown if the input is not an integer.
We'll get to handling runtime exceptions later on  
  
Great the last thing we need to do is compare the input with out secret number
```
print("You guessed " + guess + " and it was... ")
if guess > secret_number {
    println("Too high")
} else if guess < secret_number {
    println("Too low")
} else {
    println("Correct!")
}
```

If we run this code everything should work flawlessly! Assuming the input is an integer of course.  
But one thing you may have noticed is that the program exits after the first guess is made, we don't want that! We can use a while statement here! Our final code will be the following

```
use "std_rand"

var secret_number = rand_range_int(0, 100)

while true {
    var guess = read() as int

    print("You guessed " + guess + " and it was... ")
    if secret_number > number {
        println("Too high")
    } else if secret_number < number {
        println("Too low")
    } else {
        return println("Correct!")
    }
}
```