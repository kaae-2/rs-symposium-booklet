05 CLI UX

Primary commands

- `symposium-booklet build --input <file.xlsx|dir> --output <dir> [--template <dir>] [--locales en,da] [--dry-run] [--emit-parse-json] [--verbose] [--typst-bin <path>]`
- `symposium-booklet emit-typst --output <dir> [--template <dir>] [--locales en,da] [--typst-bin <path>]`
- `symposium-booklet validate <input>`

Flags and behavior

- `--input` accepts a single workbook path or a directory containing `.xlsx` files. For a directory, the parser prefers `with_ids`/`afsluttede` for abstracts and `kopi`/`grupper`/`final` for sessions; otherwise it falls back to the first two files.
- `--output` directory is wiped and recreated on `build` (dry-run reports the delete action).
- `--template` is currently used only for reporting in dry-run plans; Typst output uses the built-in minimal template.
- `--locales` default `en,da`.
- `--dry-run` validates and prints planned actions + JSON plan to stdout; no files are written.
- `--emit-parse-json` writes `output/tools_output/parse.json` and exits.
- `--verbose` enables debug logging.
- Return codes: 0 on success, non-zero on validation failure.

Examples

- Build with default template:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/`
- Dry-run:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/ --dry-run`
- Emit Typst only:
  - `symposium-booklet emit-typst --output out/ --locales en,da`
- Specify typst binary:
  - `symposium-booklet build --input data/abstracts.xlsx --output out/ --typst-bin /usr/local/bin/typst`
