# waseda-moodle-checker

Checks if there are any updates in Waseda Moodle.

It ignores

- '... ago' in 'Recent forum posts'
- '... unread posts' in forums
- status of check boxes in each activity.

## Installation

```
$ cargo install waseda-moodle-checker
```

## Usage

### Save login information

```console
$ waseda-moodle-checker login <LOGIN_ID> <PASSWORD>
```

### Remove login information

```console
$ waseda-moodle-checker logout
```

### Check updates

```
$ waseda-moodle-checker
```

### Check updates without saving login information

```
$ waseda-moodle-checker -l <LOGIN_ID> -p <PASSWORD>
```
