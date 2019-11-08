# Changelog

## Unreleased

### Added

- Support for resume arguments, via `Coroutine` and `resume_with`.
- The backing store of stack-based generators is now public (`Shelf`), so you can avoid using macros if you wish.

### Changed

- Improved panic messages (in debug builds) that try to guide you around incorrect usage.
- Stack-based generators are now "less unsafe". The lifetime of `co` is now bound by the lifetime of the generator's state, instead of `'static`. It's not fully safe yet, but it's much better.
- Improved the docs.
