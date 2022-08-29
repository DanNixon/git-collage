# git-collage [![CI](https://github.com/DanNixon/git-collage/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/DanNixon/git-collage/actions/workflows/ci.yml) [![Crates.io](https://img.shields.io/crates/v/git-collage)](https://crates.io/crates/git-collage)

A tool for selectively mirroring Git repositories.

`git-collage` was created to fill my desire to backup my own Git repos and mirror the parts of 3rd party repos I care about.
To this end the key features this tool has are:

- Ability to discover repositories (currently only from GitHub, but more can be added easily)
- Ability to filter refs that are mirrored, preventing "useless" refs being created locally (e.g. feature branches, GitHub PR merge commits)
- Is a single binary that can be scheduled via cron or systemd (personally I did not want to run a service for what is essentially a time scheduled backup job)

## Usage

See `git-collage --help` for syntax and [`examples`](./examples) for a sample configuration file.

Things to note:

- Console output below INFO level may contain sensitive information (e.g. Github tokens).
- GitHub tokens must be generated with the `repo` scope.
