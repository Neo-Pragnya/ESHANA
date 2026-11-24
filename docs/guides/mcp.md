---
title: "MCP server"
description: "eshana-mcp: let an assistant or agent compute charts with the engine on your own machine, over the Model Context Protocol."
---

# MCP server

`pip install eshana` installs `eshana-mcp`, a [Model Context Protocol](https://modelcontextprotocol.io) server that
speaks newline-delimited JSON-RPC over stdio. An assistant that supports MCP can then compute positions, charts and
calendars with the same engine as the CLI. Nothing leaves the machine: the server opens no network connection and
writes no files.

## Register it

```bash
claude mcp add eshana -- eshana-mcp          # Claude Code
```

Other clients take the same command in their stdio server settings, for example:

```json
{ "mcpServers": { "eshana": { "command": "eshana-mcp" } } }
```

## Tools

| Tool | Operation | Required |
|---|---|---|
| `compute_positions` | `positions` | none (now, tropical) |
| `create_vedic_chart` | `chart`, sidereal | `date`, `timezone` |
| `create_western_chart` | `chart`, tropical, optional dignities | `date`, `timezone` |
| `compare_charts` | `compare` | `date`, `timezone`, `a`, `b` |
| `compute_human_design` | `human_design` | `date`, `time`, `timezone` |
| `compute_numerology` | `numerology` (pythagorean, chaldean, hebrew_gematria, abjad) | none |
| `panchanga` | `panchanga` | `date`, `timezone`, `latitude`, `longitude` |
| `find_eclipses` | `eclipses` | `start_utc`, `end_utc` |
| `rise_and_set` | `rise_set` | `latitude`, `longitude`, `date`, `timezone` |
| `explain_derivation` | one attribute with its rule, the positions it came from, calculation and provenance | `date`, `timezone`, `attribute_id` |
| `eshana_request` | any operation, as a raw [request](../reference/request.md) | `request` |

Each tool returns the full envelope as text and as `structuredContent`. A refusal (an unknown zone, a missing time,
an ambiguous local time) comes back as a tool result with `isError: true` and the stable `ESH-*` code, so the model
can ask for what is missing instead of guessing.

## What it will not do

The engine never interprets: `interpretations` is always empty, and every derived value names the rule that made it.
An assistant that writes a reading from these numbers is writing its own text; the envelope's `provenance` says what
was calculated and by which rule.
