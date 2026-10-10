# GridCraft control protocol

Start the desktop app with a control port:

```sh
gridcraft --control 7979            # or GRIDCRAFT_CONTROL_PORT=7979
```

Then send newline-delimited JSON to `127.0.0.1:7979`. Every request line gets one reply line:

```json
{"id": 1, "method": "engine.execute", "params": {"command": "cell.set", "params": {"cell": "B2", "input": "=SUM(A1:A9)"}}}
{"id": 1, "ok": true, "result": {"cell": "B2", "value": "45"}}
```

Errors come back as `{"id": …, "ok": false, "error": "…"}`.

## Methods

| Method | Params | What it does |
|---|---|---|
| `engine.execute` | `{command, params}` | Runs any engine command (see `engine.commands` or `gridcraft-cli commands`) |
| `engine.commands` | | Every command: id, label, ribbon path, shortcut, params doc, enabled |
| `engine.journal` | | Commands run so far (replayable) |
| `document.inspect` | | Workbook summary: sheets, used ranges, tables, charts, names, selection, undo labels |
| `sheet.read` | `{range?, sheet?, formulas?, formatted?}` | Values (or formulas / displayed text) of a range |
| `cell.get` | `{cell?}` | One cell: value, formula, displayed text, style, merge, comment, link |
| `ui.inspect` | | UI state: ribbon tab, editor, open dialog, message, grid rect, scroll, perf |
| `ui.click` / `ui.drag` / `ui.move` | `{x, y, toX?, toY?, button?, count?, shift?, cmd?, alt?}` | Real pointer input in window points |
| `ui.cell.click` / `ui.cell.drag` | `{cell}` / `{from, to}` | Pointer input at a cell's centre (works in point mode while editing) |
| `ui.key` / `ui.text` | `{key, shift?, cmd?, alt?, ctrl?}` / `{text}` | Keyboard input (typing in the grid starts editing) |
| `ui.edit.begin` / `ui.edit.text` / `ui.edit.commit` / `ui.edit.cancel` | `{text?}` / `{move?: down\|right\|up\|left\|none}` | Drive the cell editor |
| `ui.ribbon` | `{tab}` | Switch ribbon tab |
| `ui.dialog` / `ui.dialog.set` / `ui.dialog.confirm` / `ui.dialog.cancel` | `{name}` / `{field, value}` | Open, fill in and confirm dialogs |
| `ui.screenshot` | `{path?}` | PNG of the window (base64 when no path) |
| `ui.set` | `{dark?, formulaBar?, ribbonCollapsed?}` | UI preferences |
| `ui.resize` / `ui.focus` | `{width, height}` | Window control |
| `app.open` / `app.save` / `app.quit` | `{path}` / `{path?}` | Files and lifetime |

Any other method name that is a command id (e.g. `home.bold`) runs that command.

## Language and regional format

The interface language, the formula language and the regional format are session preferences. Formulas, number
formats and `input` params are **canonical** (English function names, `,` between arguments, `.` as decimal point;
this is also what XLSX files, the journal and `engine.journal` hold); the `…Local` variants carry the text as a user of
the current language types and sees it.

| Command | Params | Result |
|---|---|---|
| `app.setLocale` | `{uiLanguage?, formulaLanguage?, regionalFormat?, useSystemSeparators?, decimalSeparator?, thousandsSeparator?}` | The `app.getInternational` payload after the change. `uiLanguage` and `regionalFormat` are `"system"` or a tag (`pt-BR`); `formulaLanguage` is `"followUi"` or `"en-US"`; the separators are one character and apply when `useSystemSeparators` is false. Applies at once: every open workbook recalculates (automatic mode) or marks its cells pending (manual mode) and the UI reloads. No dialog, not undoable. Omitted params keep their value; an invalid value changes nothing. |
| `app.getInternational` | | `{uiLanguage, formulaLanguage, regionalFormat}` (resolved tags), `decimal`, `thousands`, `list`, `arrayColumn`, `arrayRow`, `dateOrder`, `dateSeparator`, `timeSeparator`, `shortDate`, `longDate`, `r1c1: [row, column]`, `boolTrue`, `boolFalse`, `currency` and `prefs` (the six raw preferences). |
| `app.languages` | | `{languages: [{tag, nativeName, functionsTranslated}], regions: [{tag, nativeName}]}`. `functionsTranslated` is false for languages that keep English function names (ja-JP, ko-KR, zh-CN, zh-TW). |

Commands that take or return text a user types or sees:

- `cell.set`, `range.fill`: `inputLocal` (typed text, e.g. `=SOMA(1,5;2)`, `1,5`, `10/10/2026`) instead of `input`; the
  optional `locale` tag overrides the session language and region for that call.
- `cell.get`, `document.inspect`, and the commands that list names, conditional formats and validations: `formula` and
  `formulaLocal`, `numberFormat` and `numberFormatLocal`; the formula-taking commands accept the `…Local` params.
- Number-format commands accept `numberFormatLocal` (`#.##0,00`, `dd/mm/aaaa`).
- `formulas.functions`: each item has `name` (canonical) and `localName`.

The desktop app detects the system language at every start; the choice made with `app.setLocale` is saved in
`prefs.json` (`~/.config/gridcraft/` on Linux, `~/Library/Application Support/GridCraft/` on macOS, `%APPDATA%\GridCraft\`
on Windows) and in the browser's `localStorage` (`gridcraft.prefs`) for the web build.

For a headless equivalent (no window), use `gridcraft-cli mcp` or `gridcraft-cli run`;
see [`mcp.md`](mcp.md) and [`cli.md`](cli.md).
