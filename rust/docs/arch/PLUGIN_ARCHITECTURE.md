# Plugin Architecture: Three-Layer Sovereignty

## MODEL OVERVIEW

```
┌─────────────────────────────────────────┐
│          LAYER 3: LUA PLUGINS           │
│  • Append-only contributions            │
│  • 100ms CPU / 10MB RAM limits          │
│  • vo.* API bridge                      │
├─────────────────────────────────────────┤
│         LAYER 2: SYNTAX PROVIDERS       │
│  • Rust-native heuristic adapters       │
│  • Tree-sitter parsers                  │
│  • Regex engine (cached)                │
├─────────────────────────────────────────┤
│          LAYER 1: FRACTAL CORE          │
│  • Memory-mapped AST storage            │
│  • Normalized representation            │
│  • Immutable data structures            │
└─────────────────────────────────────────┘
```

## SOVEREIGNTY RULE

Plugins can ONLY:
- Register new metrics for Celestial Census
- Contribute tags to existing nodes
- Append log entries to Mission Log
- Use pre-compiled regex patterns via `vo.regex()`

Plugins CANNOT:
- Mutate or delete core-discovered data
- Access filesystem, network, or processes
- Bypass timeout or memory limits
- Load external Lua files or code

## IRON SANDBOX

The plugin sandbox enforces strict resource limits:

| Resource    | Limit   | Enforcement              |
|-------------|---------|--------------------------|
| CPU Time    | 100ms   | Interrupt handler        |
| Memory      | 10MB    | Lua allocator limit      |
| I/O         | None    | Libraries stripped       |
| Network     | None    | Libraries stripped       |

### Stripped Libraries

The following Lua standard libraries are removed:
- `io` - File I/O operations
- `os` - Operating system functions
- `debug` - Debug interface
- `package` - Module loading
- `load`, `loadfile`, `dofile` - Dynamic code loading

## vo.* API BRIDGE

Plugins interact with the Observatory through the `vo` global table:

```lua
-- API version check
assert(vo.api_version == "3.0", "API mismatch")

-- Regex bridge (uses Rust regex engine)
local pattern = vo.regex("fn\\s+(\\w+)")
local count = pattern("fn main() { fn helper() {} }")
-- count = 2

-- Pre-compiled patterns
local rust_fn = vo.patterns.rust_fn
local python_def = vo.patterns.python_def

-- Logging
vo.log("info", "Plugin initialized")

-- Metric registration
vo.register_metric("todo_count", function(ast)
    local count = 0
    for _, comment in ipairs(ast.comments or {}) do
        if string.find(comment.text, "TODO") then
            count = count + 1
        end
    end
    return {
        value = count,
        confidence = 0.9,
        explanation = "Found " .. count .. " TODO comments"
    }
end)

-- Tag contribution
vo.contribute_tag("src/main.rs:42", "needs-review")
```

## PLUGIN MANIFEST

Plugins are discovered via `manifest.json`:

```json
{
    "vo_api_version": "3.0",
    "plugins": [
        {
            "name": "todo-counter",
            "file": "todo_counter.lua",
            "enabled": true,
            "priority": 100
        }
    ]
}
```

### Discovery Paths

1. `.vo/plugins/` - Project-local plugins
2. `~/.config/vo/plugins/` - User-global plugins

## DETERMINISM GUARANTEE

Plugin contributions are merged using `BTreeMap` to ensure:
- Consistent ordering across runs
- Byte-identical output
- Reproducible Celestial Census results

## GRACEFUL DEGRADATION

When plugins are disabled (`--no-plugins` or feature not compiled):
- Zero performance overhead
- All core functionality intact
- Clean fallback messages in Mission Log

## SECURITY CONSIDERATIONS

1. **Sandboxing**: All plugin code runs in isolated Lua VM
2. **Resource Limits**: Hard caps on CPU and memory
3. **No Side Effects**: Append-only data model
4. **Version Pinning**: API version checked before load
5. **Timeout Enforcement**: Interrupt handler for runaway plugins

## EXAMPLE PLUGIN

```lua
-- ~/.config/vo/plugins/complexity_checker.lua
-- A plugin that flags overly complex functions

local function check_complexity(ast)
    local warnings = {}

    for _, func in ipairs(ast.functions or {}) do
        local nesting = func.max_nesting or 0
        local params = #(func.parameters or {})

        if nesting > 4 then
            table.insert(warnings, {
                file = func.file,
                line = func.line,
                message = "Deep nesting detected: " .. nesting .. " levels"
            })
        end

        if params > 5 then
            table.insert(warnings, {
                file = func.file,
                line = func.line,
                message = "Too many parameters: " .. params
            })
        end
    end

    return warnings
end

-- Register with Observatory
vo.register_metric("complexity_warnings", function(ast)
    local warnings = check_complexity(ast)
    return {
        value = #warnings,
        confidence = 1.0,
        explanation = #warnings .. " complexity warnings found",
        details = warnings
    }
end)
```
