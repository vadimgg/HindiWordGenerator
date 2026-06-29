# `lingo viewer`

Serve the local web UI over the current library. This is also the **default**
command — running `lingo` with no subcommand opens the viewer.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Serve the local web viewer.

Usage: lingo viewer [OPTIONS]

Options:
      --port <PORT>   Port to listen on [default: 4321]
  -h, --help          Print help
```

Help colors: `viewer`/flags **green**, `<PORT>` **yellow**, headers **bold cyan**.

## What it's for

The viewer is a **third actor on the same loop** as the CLI and coding agents (see
[`workflows.md`](../workflows.md), Workflow C). Its "Generate" buttons create the
same runs, its textareas are another place to paste a reply, and "Apply" calls the
same use case. Anything done in the UI is immediately visible to the CLI, and vice
versa — there is no separate data path.

Use the viewer for browsing, reordering, curating (marking sentences `active`), and
quick edits; use the CLI for the bulk extract → enrich → audio → publish pipeline.

## Example

```bash
lingo viewer --port 4321
# or simply:
lingo
```

Sample output:

```
Viewer   serving my-hindi-library
  http://localhost:4321

Press Ctrl-C to stop.
```

Colors: `Viewer` heading **bold cyan**; the URL **cyan**; the hint **dim**.

## See also

[`status`](./status.md) · [`edit`](./edit.md) · [`workflows.md`](../workflows.md)
