
# Corrator
[![crates.io](https://img.shields.io/crates/v/corrator?label=latest)](https://crates.io/crates/corrator) 
[![Documentation](https://docs.rs/corrator/badge.svg?version=latest)](https://docs.rs/corrator/latest) 
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/corrator.svg) 
[![Dependency Status](https://deps.rs/crate/corrator/latest/status.svg)](https://deps.rs/crate/corrator/latest)
![downloads](https://img.shields.io/crates/d/corrator.svg) 

Keep tabs on the corrosion in your docker containers.

## What is it?

Corrator is a command line tool for querying docker containers and requesting version numbers for
apps in them.


## Why would I want that?

If you have to maintain lots of containers, each with their own set of applications and dependencies, you may be familiar with how difficult it can be to maintain currency. You don't want to let your dependencies go out of their support life-cycles, but you also don't want to have to update a master list somewhere any time you push an update.

Corrator lets you define a set of containers, each with a list of important dependencies. Running corrator will pull down the container, bash into it, and run a version command on each one. Then it spits out what it finds in minimal form.

For dependencies which happen to be tracked by the excellent endoflife.date service you can optionally ask corrator to look up the current end of life date *for the version currently installed* and tell you that too.

## Installing Corrator

### From Crates.io

#### Install Rust

See the [rust docs](https://doc.rust-lang.org/stable/book/ch01-01-installation.html) for this.

#### Install Corrator

You can install corrator straight from crates.io with the following:

```sh
cargo install corrator
```

## Using Corrator

Assuming you have a valid config file (see below), you can simply run `corrator` from the command line.

```sh
corrator
```

For additional options, see `corrator --help`.

## Configuring Corrator

The heart of corrator is a configuration directory featuring two files:

### applications.toml

```toml
[bash]

# The actual command to run to get a version
version_command = "bash --version"

# Command's version format as a regex
#   "version" named group is mandatory
version_regex = '''GNU bash, version (?P<version>[0-9.]+)'''

# Optional for endoflife.date support
[eol]
# The "product name" as it exists in endoflife.date
# yes, I'm aware bash isn't actually on endoflife.date
product_name = "bash"

# regex for which parts of version endofdate is looking for
#   e.g., Rails only wants version in X.X format
version_regex = '''.+'''
```

### containers.toml

```toml
[ubuntu]

# The docker registry full path
path = "ubuntu"

# An array of applications to be queried
#   These must be defined in this file
apps = [ "bash" ]

# An array of tags
#   These can be anything sting you want
#   You can filter them with `--tag` and `--filter`
#   See `corrator --help` for more detail
tags = [ "mytag1", "mytag2" ]
```

Corrator will look for these files in the following locations, in order:

1.  Your system's user config location (see `corrator --help` to find this path)
3.  Using the flag `-c path_to_directory`

There is also an `examples` in this repository to get you started.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
