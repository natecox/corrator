
# Corrator

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

The heart of corrator is a simple `toml` file with the following schema:

```toml
[applications.bash]
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


[containers.ubuntu]
# The docker registry full path
path = "ubuntu"

# An array of applications to be queried
#   These must be defined in this file
apps = [ "bash" ]
```

Corrator will look for this file in the following locations, in order:

1.  Your system's user config location (see `corrator --help` to find this path)
2.  Using the environment variable `CORRATOR_CONFIG_PATH`
3.  Using the flag `-c path_to_toml`

There is also an example file in this repository to get you started.
