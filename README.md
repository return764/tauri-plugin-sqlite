# Tauri Plugin sqlite

[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Tauri plugin for sqlite database based on Sqlx

- Consistent with the [official sql plugin](https://v2.tauri.app/zh-cn/plugin/sql/) api, but only supports sqlite
- Support Sqlite extension

## Installation

### Rust

```bash
cargo add tauri-plugin-sqlite
```

### Webview

```bash
npm install tauri-plugin-sqlite-api
# or
yarn add tauri-plugin-sqlite-api
```

## Usage

### Initialize plugin

Configure plugin in `tauri.conf.json`：

```json
{
  "plugins": {
    "sqlite": {
      "preload": ["sqlite:test.db"] // optinal：preload database
    }
  }
}
```

### Migration in rust

```rust
use tauri_plugin_sqlite::{Builder, Migration, MigrationKind};

fn main() {
    tauri::Builder::default()
        .plugin(
            Builder::default()
                .add_migrations(
                    "sqlite:test.db",
                    vec![
                        Migration {
                            version: 1,
                            description: "create_users_table",
                            sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
                            kind: MigrationKind::Up,
                        },
                    ],
                )
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend api

```typescript
import Database from "tauri-plugin-sqlite-api";

type User = {
    id: number,
    name: string
}

// load database
const db = await Database.load({
  db_url: 'sqlite:test.db',
  extensions: ["some extension"]
});

// execute
const result = await db.execute('INSERT INTO users (name) VALUES (?)', ['John']);
console.log(result); // { rowsAffected: 1, lastInsertId: 1 }

// select
const rows = await db.select<User>('SELECT * FROM users WHERE name = ?', ['John']);
console.log(rows); // [{ id: 1, name: 'John' }]

// close
await db.close(db);
```

### Update Plan
- [ ] support more sqlite options
- [ ] support orm?
- [ ] support database transaction
