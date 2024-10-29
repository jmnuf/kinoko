# Kinoko Mushroom

For the uninformed, the title is mushroom mushroom essentially.
Tiny program for compiling small rust projects.
Initially written cause I don't understand cargo and `rustc` is just fine for most of my uses of rust, but too long to write the command even once (I'm pretty lazy). So I wrote a program that will just run the `rustc` command for me. It's not extremely crazy of a project it essentially just runs `rustc -o build/program src/main.rs` which honestly could be a bash script, but I don't want to write a bash script for all my minor utilities written in rust. Also I am not a rustacean, thank you very much.

Kinoko only is operational on code that does **NOT** use dependencies outside of the standard library.
> NOTE:
> Not sure how obvious it is to some but it is required to have rust installed with `rustc` included in your path.

## kinoko.🍄
The `kinoko.🍄` file is the basis on how the mini-project is compiled in a very naive sense. All it keeps track of as of now is the target name and the entry file. ~~If the file doesn't exist it will be created by attempting to find where a `fn main()` might exist in the current directory or the `src` subdirectory if it exists to figure out where's the entry file and proceeds to use the current directory's name as the output file name.~~
> NOTE:
> Before the kinoko program would auto-generate the kinoko file if not found by searching for an entry point. This functionality is momentarily removed and will be brought back in the future as a separate command probably.
The file holds the following structure:
```
root: src/main.rs
head: build/output
```
With `root` referring to the entry file and `head` referring to the output file.

## Run/Germinate
To make the mushroom germinate an executable all that's needed is to run the following:
```console
$ kinoko build
```
This will automatically call onto `rustc` and provide what are the entry file and the output file based on `kinoko.🍄` file.
Will approximately do the following:
```console
$ rustc -o build/WHY_IS_THERE_CODE src/main.rs
```

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
- [ ] Be able to clear out kinoko and transition to cargo
