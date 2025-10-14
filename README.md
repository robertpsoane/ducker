### Custom Container Display

By default, Ducker shows all 7 container columns (ID, Image, Command, Created, Status, Ports, Names). However, you can customize which fields are displayed using two different configuration approaches for maximum flexibility:

#### Template Format (Advanced)

Use a Go-template style format string, similar to Docker's `ps --format` flag:

```yaml
format: "table {{.ID}}\t{{.Names}}\t{{.Status}}"
```

This will display only short container IDs, names, and status columns with proper spacing.

**Available Fields:**
- `{{.ID}}` - Container ID (shown as short 12-character format)
- `{{.Image}}` - Container image
- `{{.Command}}` - Container command
- `{{.Created}}` - Creation timestamp
- `{{.Status}}` - Container status
- `{{.Ports}}` - Port mappings
- `{{.Names}}` - Container names

**Advanced Examples:**

Show only essentials:
```yaml
format: "table {{.ID}}\t{{.Names}}\t{{.Status}}"
```

Show image and status:
```yaml
format: "table {{.Image}}\t{{.Status}}\t{{.Names}}"
```

Show minimal columns:
```yaml
format: "table {{.Names}}\t{{.Status}}"
```

Reorder columns:
```yaml
format: "table {{.Status}}\t{{.Names}}\t{{.ID}}"
```

#### Boolean Display Columns (Simple)

For easier configuration without template syntax, use boolean toggles:

```yaml
display_columns:
  containers:
    id: true
    image: false
    command: true
    created: false
    status: true
    ports: false
    names: true
```

This shows ID, Command, Status, and Names columns in standard Docker order.

**Configuration Priority:**

1. `format` (highest priority - if set, overrides boolean config)
2. `display_columns.containers` (boolean toggles)
3. Default (all columns if neither is specified)

Leave both `format` and `display_columns` unset or set to `null` to use the default display with all columns.
