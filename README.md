# Cherry

A developer-centric, type-safe, ultra-ergonomic SQLite ORM for Rust, built on top of `rusqlite`.

`cherry` rejects the "one ORM to rule them all" philosophy. By focusing exclusively on **SQLite**, it avoids the heavy architectural compromises, bloated generic parameters, and unreadable compile errors found in multi-database ORMs. It embraces Rust's type system to provide an API that reads exactly like raw SQL while eliminating the most common database footguns—like column typos, nullability mismatches, and incorrect parameter binding—at compile time, without locking you into a rigid type-gymnastics prison.


------

## The Core Philosophy

1. **The DB is the Subject:** Every operation begins with the database instance (`db.select`, `db.insert`). Entities are pure, lightweight data structures without hidden state or active-record side effects.
2. **Visual Mapping, No Re-invention:** The DSL is designed to map directly to your mental model of SQL. `User::age.gt(18)` produces `age > 18` seamlessly. No complex object-graph abstractions.
3. **Zero Default Contagion:** Partial column scans shouldn't force your fields to implement `Default` or be wrapped in risky `Option` wrappers. `cherry` introduces precise compile-time projection mapping.
4. **Compile-Time Typestate Patterns:** Builders use Rust's type states to modify method availability and return types dynamically. An insert statement with a `returning` clause changes the signature of `.execute()` at compile time.

------

## 1. Defining Models & DTOs

Deriving `Model` automatically exposes your struct to the query engine and generates lowercase static column constants (e.g., `User::id`, `User::name`) for compile-time safe DSL filtering.

```rust
use cherry::{FromRow, Model};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSettings {
    pub theme: String,
    pub notifications: bool,
}

#[derive(Model)]
#[orm(table = "users")] // Customizes the table name; defaults to snake_case of the struct
pub struct User {
    #[orm(autoincrement)] // Tells the compiler to skip this field during standard inserts
    pub id: i64,
    
    #[orm(column = "user_name")] // Overrides database column naming
    pub name: String,
    
    pub age: i32,
    pub role: String,
    pub is_active: bool, // Automatically maps to SQLite INTEGER (0 or 1)
    
    // The framework handles JSON proxying entirely at compile time via transparent macros.
    // No ugly Newtype wrappers needed, and no Trait Coherence violations.
    // Your entity remains a pure data carrier (Needed enable "json" feature).
    #[orm(json)]
    pub members: Vec<String>,
    
    #[orm(json)]
    pub settings: UserSettings,
    
    #[orm(ignore)] // Excluded from all database schemas and mapping
    pub runtime_cache: Vec<String>,
}

// A Data Transfer Object (DTO) for partial queries.
// Implements `FromRow` but NOT `Model` - completely read-only.
#[derive(FromRow)]
pub struct UserProfileDTO {
    pub id: i64,
    #[orm(column = "user_name")]
    pub name: String,
    pub role: String,
}
```

------

## 2. Querying (Select)

### Simple Queries & Method Idempotency

Chain selectors intuitively. Calling `.filter()` multiple times automatically combines them with `AND` expressions via internal AST nodes, or you can use native bitwise `&` and `|` operators to form explicit logical groups.

```rust
// Fetch all active adults named Alice or Bob
let users: Vec<User> = db.select::<User>()
    .filter((User::name.eq("Alice") | User::name.eq("Bob")) & User::age.gte(18))
    .filter(User::is_active.eq(true)) // Automatically appended using AND under the hood
    .order_by(User::age.desc())
    .limit(10)
    .load()?; // Returns Vec<User>
```

### Advanced Projections (DTOs & Tuples)

Avoid loading massive rows into memory. Select specific columns and map them either into a lightweight DTO or directly into an anonymous Rust tuple.

*Ergonomics Note: If you only need a single column, pass it directly. For multiple columns, wrap them in a tuple.*

```rust
// Ergonomics: Single column requires NO tuple wrapping
let user_ids: Vec<i64> = db.select::<User>()
    .columns(User::id) 
    .filter(User::age.gt(18))
    .load_as::<i64>()?;

// Scenario A: Load multiple fields directly into a DTO struct
let profiles: Vec<UserProfileDTO> = db.select::<User>()
    .columns((User::id, User::name, User::role)) // Multiple columns wrapped in a tuple
    .filter(User::age.gt(30))
    .load_as::<UserProfileDTO>()?;

// Scenario B: Load multiple fields into an anonymous tuple on the fly
let rows: Vec<(i64, String)> = db.select::<User>()
    .columns((User::id, User::name))
    .filter(User::role.eq("admin"))
    .load_as::<(i64, String)>()?; // Auto-mapped via macro-implemented FromRow for Tuples
```

### Aggregate, Distinct & Grouping

```rust
// Perform quick scalar counting
let total_admins: usize = db.select::<User>()
    .filter(User::role.eq("admin"))
    .count()?;

// Complex analytical grouping
let stats = db.select::<User>()
    .distinct()
    .columns("role, count(1) as total") // Supports raw fragments when needed
    .group_by(User::role)
    .having(User::age.gt(18))
    .load_as::<RoleStatsDTO>()?;
```

### Type-Safe JOINs (The Grail)

Multi-table joins are powered by a predictable column-aliasing mechanism under the hood (table.column as "table.column"). By isolating fields with qualified names, cherry seamlessly unpacks rows into structured tuples without complex type-level coordinate matching, keeping macro overhead low and compile times exceptionally fast.

You can seamlessly mix `INNER JOIN` and `LEFT JOIN`. If a `LEFT JOIN` fails to find a match, the `Option<M>` element decisively resolves to `None`.

```rust
// INNER JOIN returning paired strong entities
let rows: Vec<(User, Role)> = db.select::<User>()
    .inner_join::<Role>(User::role.eq(Role::name))
    .filter(Role::permission_level.gt(5))
    .load_as::<(User, Role)>()?;

// A comprehensive mixed JOIN: 
// Fetch users, their mandatory roles, and optional company details in one highly-structured tuple.
let rows: Vec<(User, Role, Option<Company>)> = db.select::<User>()
    .inner_join::<Role>(User::role_id.eq(Role::id))
    .left_join::<Company>(User::company_id.eq(Company::id))
    .filter(Role::permission_level.gt(5))
    .filter(User::is_active.eq(true))
    .load_as::<(User, Role, Option<Company>)>()?;
```

#### ⚠️ Limitation Note: Self Joins & Table Aliasing
Because `cherry` currently isolates columns using their native table names (e.g., `users.id`), performing a `SELF JOIN` (joining a table onto itself, such as matching a user to their manager) will cause column name conflicts in the underlying mapper.

If your query requires a self-join, please temporarily bypass the DSL and use the `db.raw()` escape hatch:

```rust
// Temporary workaround for Self Joins using Raw SQL
let rows: Vec<(User, UserProfileDTO)> = db.raw(
    "SELECT u.*, m.user_name as m_name, m.role as m_role \
     FROM users u \
     INNER JOIN users m ON u.manager_id = m.id"
).load_as::<(User, UserProfileDTO)>()?;
```
Full DSL support for dynamic table aliasing (`.alias()`) is planned for a future release(Maybe).

------

## 3. Inserting (Single, Batch & Upsert)

### Unified Insertion & Auto-Chunking

Whether inserting a single reference, an array slice, or a large heap-allocated `Vec`, the engine accepts them through the identical `AsRows` trait boundary. If a batch contains more variables than SQLite's parameter limit (32,766 parameters), **`cherry` automatically and transparently splits it into multiple sequential queries under the hood.**

```rust
let new_user = User { id: 0, name: "Charlie".into(), age: 25, /* ... */ };
let batch_users = vec![user_a, user_b, user_c]; // Massive dataset

// Single row execution
let affected: usize = db.insert(&new_user).execute()?;

// Massive multi-row execution (Auto-chunked to prevent SQLite parameter limits error)
let total_inserted: usize = db.insert(&batch_users).execute()?;
```

### Smart Upserts (`ON CONFLICT`)

Like the selection projection API, `on_conflict` is fully polymorphic. Pass a single column directly, or pass a tuple of mixed types to match a composite unique index.

```rust
// Single column conflict target requires NO tuple wrapping
db.insert(&new_user)
    .on_conflict(User::id)
    .update(User::role.set("admin"))
    .execute()?;

// Advanced compound index ON CONFLICT DO UPDATE SET configuration
let affected = db.insert(&new_user)
    .on_conflict((User::name, User::role)) // Composite unique key targets wrapped in a tuple
    .update(User::age.set(26))             // Standard overwrite variable binding
    .update(User::is_active.excluded())    // Generates `is_active = excluded.is_active` safely
    .filter(User::age.lt(30))              // Context-aware AST clause aware of the upsert environment
    .execute()?;
```

### Typestate `.returning()` Clauses

By calling `.returning()`, the builder changes its compile-time type signature. The terminal `.execute()` method transitions from returning a `usize` (affected rows) to returning a strongly-typed `Vec` of requested elements.

```rust
// State Transition: Grab newly generated auto-increment IDs
let assigned_ids: Vec<i64> = db.insert(&batch_users)
    .returning(User::id) // Single column
    .execute()?; // Returns Result<Vec<i64>> due to Typestate pattern

// State Transition: Grab multi-column tuple metrics on creation
let metadata: Vec<(i64, String)> = db.insert(&new_user)
    .returning((User::id, User::role)) // Multiple columns via tuple
    .execute()?; // Returns Result<Vec<(i64, String)>>
```

------

## 4. Updating & Deleting

`cherry` protects you from catastrophic accidents. Executing an update builder without calling `.set()` intercepts the execution pipeline to prevent invalid database transitions.

```rust
// Target fields and update securely
let updated_count: usize = db.update::<User>()
    .set(User::role.set("moderator"))
    .filter(User::id.eq(42))
    .execute()?;

// Delete with standard conditions
let deleted_count: usize = db.delete::<User>()
    .filter(User::is_active.eq(false))
    .execute()?;
```

Both actions natively support the unified `.returning()` and `.returning_as::<DTO>()` state pipelines exactly like the insertion architecture.

------

## 5. The Escape Hatch: Raw SQL

Even with a powerful DSL, sometimes you just need to write raw SQL (e.g., recursive CTEs, specialized SQLite extensions, or PRAGMA commands). `cherry` provides a `Raw` query builder that perfectly integrates with the rest of the ecosystem—including the `FromRow` projection magic!

```rust
// Execute a raw UPDATE/DELETE with simple bindings
let affected = db.raw("UPDATE users SET age = age + 1 WHERE status = ?")
    .bind("active")
    .execute()?;

// Execute a raw SELECT and map directly into a Tuple or DTO seamlessly!
let active_users: Vec<(i64, String)> = db.raw("SELECT id, user_name FROM users WHERE age > ?")
    .bind(18)
    .load_as::<(i64, String)>()?;

// Dynamic bindings using `bind_all` (Perfect for dynamic `IN` clauses)
let ids = vec![1, 2, 3];
let placeholders = vec!["?"; ids.len()].join(", ");
let sql = format!("SELECT * FROM users WHERE id IN ({})", placeholders);

let users: Vec<User> = db.raw(sql)
    .bind_all(ids)
    .load_as::<User>()?;
```

------

## 6. Connection Management & Pooling

`cherry` features a flexible `DatabaseBuilder` providing fine-grained optimization control. High-concurrency sqlite parameters can be configured fluidly out of the box.

### Single Connection Mode (Default)

Ideal for lightweight applications, command-line tools, or embedded environments.

```rust
use cherry::db::{Database, DatabaseBuilder};

// Basic instant opening
let db = Database::open("app.db")?;

// Or use advanced customization configuration via builder
let db = DatabaseBuilder::new("app.db")
    .add_pragma("PRAGMA foreign_keys = ON;")
    .build()?;
```

### Highly Concurrent Connection Pooling

By enabling the `pool` feature, `cherry` transitions into an optimized `r2d2` shared pooling architecture. Calling `.with_default_pragmas()` automatically injects industry-proven settings (`WAL` mode, `NORMAL` synchronous block, and busy timeout handlers) to maximize concurrent reader performance.

```rust
// Enable pooled variants fluidly (requires cargo "pool" feature)
let db = DatabaseBuilder::new("app.db")
    .with_default_pragmas() // Enables WAL mode, synchronous=NORMAL, busy_timeout=5000ms
    .pooled(10)             // Sets maximum connection pool size
    .build()?;
```

------

## 7. Transaction Processing

`cherry` abstracts the complexities of scope management via an automated closure context pipeline. If your closure block completes successfully with `Ok(())`, it auto-commits; if any error bubbles up via `?`, it triggers an automatic rollback.

### Shared CRUD Ergonomics

Because both `Database` and `Transaction` bind queries through a unified internal `Executor` design, **their public query APIs are 100% identical.**

### Safe Compile-Time Borrowing (Single vs Pooled)

`cherry` leverages Rust's conditional feature flag mechanics to preserve absolute performance efficiency depending on your deployment choice:

- **Without `pool` feature:** `.transaction()` requires `&mut self` to guarantee exclusive single-thread isolation.
- **With `pool` feature:** `.transaction()` accepts `&self`. Spawning a transaction thread simply borrows a scoped node from the pool without locking down the parent database coordinator.

```rust
// Transaction management block via closure execution
db.transaction(|tx| {
    // Spawns query operations directly on the transaction context
    // The public API here is completely identical to calling db.insert / db.update
    let user_id = tx.insert(&new_user).returning(User::id).execute()?[0];
    
    tx.update::<User>()
        .set(User::role.set("verified"))
        .filter(User::id.eq(user_id))
        .execute()?;
        
    Ok(()) // Automatically commits here, or rolls back if an Err is returned
})?;
```

------

## 8. Custom Type Mapping (The `Type` Trait)

One of `cherry`'s most powerful features is how seamlessly you can map your own Rust types (like Enums or Domain-Specific wrappers) to the database. Because `cherry` completely isolates the underlying driver, **you never have to touch `rusqlite`'s complex `ToSql` or `FromSql` traits.**

Simply implement the `Type` trait and map your custom type to one of the 5 fundamental SQLite storage classes (`Integer`, `Real`, `Text`, `Blob`, `Null`).

```rust
use cherry::{Error, Result};
use cherry::types::{Type, Value};

#[derive(Debug, PartialEq)]
pub enum Status {
    Pending,
    Active,
    Banned,
}

// Implement Type mapping
impl Type for Status {
    fn to_db(&self) -> Result<Value> {
        let int_val = match self {
            Status::Pending => 0,
            Status::Active => 1,
            Status::Banned => 2,
        };
        Ok(Value::Integer(int_val)) // Stored as SQLite INTEGER
    }

    fn from_db(value: Value) -> Result<Self> {
        match value {
            Value::Integer(0) => Ok(Status::Pending),
            Value::Integer(1) => Ok(Status::Active),
            Value::Integer(2) => Ok(Status::Banned),
            _ => Err(Error::TypeMismatch("Expected integer (0,1,2) for Status".into())),
        }
    }
}

// Use it directly in your Models!
#[derive(Model)]
pub struct Order {
    pub id: i64,
    pub status: Status, // Works out-of-the-box!
}

// Enjoy full type-safety in your DSL
db.update::<Order>()
    .set(Order::status.set(Status::Active))
    .filter(Order::status.eq(Status::Pending)) // The compiler ensures you pass a `Status` enum!
    .execute()?;
```

------

## 9. Optional Cargo Features

`cherry` handles native primitives out-of-the-box (`bool`, `Vec<u8>`). Richer type mappings are isolated behind explicit optional features to maintain rapid compilation times and clean dependency trees.

| **Feature Name** | **Target Crate** | **Database Representation** | **Defending Domain Realities**                               |
| ---------------- | ---------------- | --------------------------- | ------------------------------------------------------------ |
| `pool`           | `r2d2`, `r2d2_sqlite`   | `N/A` | Provides stable pooled connection access insulated from per‑request initialization overhead. |
| `json`           | `serde_json`     | `TEXT`                      | Serializes flexible object notation configurations directly into standardized text elements. |
| `chrono`         | `chrono`         | `TEXT`                      | Formats to industry-standard ISO 8601 (RFC 3339) strings, allowing quick native database dictionary evaluations. |
| `time`           | `time`           | `TEXT`                      | Maps `time::OffsetDateTime` cleanly via structural RFC 3339 constraints. |
| `jiff`           | `jiff`           | `TEXT`                      | Integrates modern time mechanics (`jiff::Timestamp`) into string storage nodes. |
| `uuid`           | `uuid`           | `BLOB (16 bytes)`           | Enforces memory efficiency on disk, yet remains broad enough to leniency parse string fields on legacy reads. |
| `url`            | `url`            | `TEXT`                      | Maps strict validated web resource paths directly onto database nodes. |

Activate your ecosystem extensions inside your `Cargo.toml`:

```toml
[dependencies]
cherry = { version = "0.2.0", features = ["pool", "json", "chrono", "time", "jiff", "uuid", "url"] }
```

---

## 10. Limitations & Design Trade-offs

To keep `cherry` incredibly fast and focused, we intentionally made some hard architectural choices:

* **Purely Synchronous:** There is no `async` support. SQLite is a local, embedded database. Wrapping its native C-bindings in an asynchronous runtime adds unnecessary overhead, bloats the memory footprint, and slows down compilation. `cherry` is strictly synchronous by design, making it perfect for edge computing, desktop apps, and WASM.
* **No Schema Migrations:** `cherry` is exclusively a query and mapping engine. It does not manage your table creation or schema versions. We recommend pairing it with dedicated, specialized migration tools like [`refinery`](https://github.com/rust-db/refinery) or [`barrel`](https://github.com/rust-db/barrel).

---

## 11. MSRV & Edition

`cherry` relies on modern Rust features and strictly requires the **2024 Edition**.
The Minimum Supported Rust Version (MSRV) is **`1.85.0`**.

---

## 12. License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.