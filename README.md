# Classroom Helper

A utility to ease the usage of GitHub Classroom.

It has the following features:

1. Fetch and update students' repos in parallel.
2. Grade students by running python3/bash scripts in parallel.
3. Check `git diff` and `git log` for whitebox grades.
4. Export all grades in UTF-8 csv format.
5. Easy to read configuration file using TOML.

See `template.toml` for configuration example. You can run `cargo run -- -h` for command line help.

Key bindings in tui:

       H J K L: navigate between panels
       j k: scroll in panels
       f F: fetch selected(f)/all(F) students
       g G: grade blackbox for selected(g)/all(G) students
       s d: save(s)/diff(d) results
       [num]+b w: set blackbox(b)/whitebox(w) grade manually
       r: repeat last grade for current student
       t: bump template repo to newest version
       c: edit comment

It expects grading scripts to output a JSON like the following format:

```json
{"grade": 100.0}
```