# Classroom Helper

A utility to ease the usage of GitHub Classroom.

It has the following features:

1. Fetch and update students' repos in parallel.
2. Grade students by running python3 scripts in parallel.
3. Check `git diff` and `git log` for whitebox grades.
4. Export all grades in UTF-8 csv format.
5. Easy to read configuration file using TOML.

See `template.toml` for configuration example. You can run `cargo run -- -h` for command line help.
