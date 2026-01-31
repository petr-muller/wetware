# Bootstrapping the **wetware** project

The goal of this project is to build software that will allow me to track what I like to call _thoughts_. Thoughts are
brief snippets of information that I want to capture, organize, process and review later. A typical thought is a single
sentence. Sometimes it can be two or three, almost never more. Thoughts refer to 'entities' which are subjects or
'objects' of the thought. An entity can be a person, a place, an event, a concept, etc. Thoughts are associated with a
date and have a unique identifier, used by the user to refer to them. 

## Initial project elements

- The project will be implemented in modern Rust
- The project will contain a command line interface binary called `wet`, which will be the main user entry point.
- The binary will be a very thin wrapper around other modules that contain all logic
- The `wet` binary will use a command paradigm with a hierarchy of subcommands, implemented using the `clap` crate and derived macros
- There will be a module that will contain all the command line interface logic
- There will be a module that will contain data structures and logic for the business domain of the project. Initially,
  the only domain concept is a thought, which is simply a string associated with a unique identifier (for now treated
  as internal).
- There will be a module that will contain all logic for managing persisted data. The module will offer a clear abstract
  API to manipulate data in storage and will contain a single implementation that uses an SQLite database. The API should
  allow for easy swapping of the implementation in the future, if needed.
- There will be a module that will contain logic for reading user input and translating it into the domain model.

### Examples

```console
$ wet add "This is a thought"
$ wet add "This is another thought"
$ wet thoughts
This is a thought
This is another thought
```

## Tests

- The project will have CLI integration using the `assert_cmd` crate
- The project will have unit tests for all major functionality, targeting 90%+ code coverage

## Source control

- The project will use conventional commits
