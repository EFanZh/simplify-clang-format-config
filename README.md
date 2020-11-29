# simplify-clang-format-config

`simplify-clang-format-config` can be used to generate a simplified [ClangFormat](https://clang.llvm.org/docs/ClangFormat.html) configuration base on an existing configuration.

## Usage

```text
simplify-clang-format-config [--clang-format-executable <clang-format-executable>] [config-file]
```

If `--clang-format-executable` option is not specified, `clang-format` will be used.

If `config-file` argument is not specified, the standard input will be used.
