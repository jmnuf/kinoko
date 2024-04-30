# Kinoko Mushroom

For the uninformed, the title is mushroom mushroom essentially.
Tiny program for compiling small rust projects.

Cargo does a lot and I don't get it, nor do I care to understand much about it as of the writing of this, so in retaliation for simplicity I wrote a thin wrapper over rustc for building small CLI applications that could be written in bash if I knew how to write a for loop in it.
It's unclear whether this project's scope will grow.

## kinoko.🍄
The `kinoko.🍄` file is the basis on how the mini-project is compiled in a very naive sense. All it keeps track of as of now is the target name and the entry file. If the file doesn't exist it will be created by attempting to find where a `fn main()` might exist in the current directory or the `src` subdirectory if it exists to figure out where's the entry file and proceeds to use the current directory's name as the output file name.
The file holds the following structure:
```
root: src/main.rs
head: build/output
```
With `root` referring to the entry file and `head` referring to the output file.

## Init a project:
You can initialize a minimal project by doing the following command:
```console
$ kinoko init <project-name>
```
> **NOTE**:
> If you want to create it at the current directory, you can omit the project name.

This will create the following structure:
```
 project/
   |- build/
   |
   |- src/
   |  |- main.rs
   |
   |- kinoko.🍄
```

The `main.rs` file will have a base hello world setup which also prints the arguments passed onto the program.

TODO:
- [ ] Be able to change the target name/path through the CLI
- [ ] Be able to change the source through the CLI
- [ ] Be able to run tests like cargo
- [ ] Be able to clear out kinoko and setup cargo for transitioning
