05 CLI UX

Primary command

- `symposium-booklet build --input <file.xlsx> --output <dir> [--template <dir>] [--locales en,da] [--dry-run] [--verbose] [--typst-bin <path>]`

Subcommands (future)

- `symposium-booklet validate <input>` — validate Excel files and print schema errors without writing files
- `symposium-booklet render-typst <manifest> --locale en` — generate typst file only
- `symposium-booklet watch` — watch input files and rebuild on change (future)

Flags and behavior

- `--input` accepts a single workbook path or directory containing workbooks. If directory, process all `.xlsx` files found.
- `--output` directory is created if it doesn't exist.
- `--template` overrides the built-in starter typst template directory.
- `--locales` default `en,da`.
- `--dry-run` will validate and print the actions without writing files.
- `--verbose` enables debug logging.
- Return codes: 0 on success (files emitted). Non-zero on validation failure.

Examples

- Build with default template:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/`  
- Dry-run:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/ --dry-run`  
- Specify typst binary:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/ --typst-bin /usr/local/bin/typst`