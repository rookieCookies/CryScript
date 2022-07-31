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
<br>
# Common Concepts
## Variables
Variables are containers for storing different data values  
In CryScript there are a few types of variables  
* Strings   |  stores a text such as "Hello World!"  
* Integers  |  stores a whole number such as 987 or -123
* Float     |  stores a whole number such as 987 or -123
* Function  |  stores a function reference to be called later<br>

### Declaring (Creating) variables
#### Syntax: `var variable_name = value`  
Let's look at that word by word,
* "var"  is just a keyword indicating that this is a variable declaration
* "variable_name" is the name of the variable such as (x or name)
* the equal sign is used to assign a value to the variable
* and then the value is the actual value we want the variable to be

Don't worry about trying to memorize all of these since you'll get used to it as we go on with examples, let's look at some examples shall we?   
   
#### Examples

1. Declare a variable called "message" and then assign it to "Hi!"  
    >var message = "Hi!"
2. Declare a variable called "message" and then assign it to 12  
    >var message = 12
3. You can also declare a variable and assign it later  
    >var message  
    >message = "Yep!"
4. Or even override an existing value
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
