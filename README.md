# cherry

A developer-centric, type-safe, ultra-ergonomic SQLite ORM for Rust, built on top of `rusqlite`.

`cherry` rejects the "one ORM to rule them all" philosophy. By focusing exclusively on **SQLite**, it avoids the heavy architectural compromises, bloated generic parameters, and unreadable compile errors found in multi-database ORMs. It embraces Rust's type system to provide an API that reads exactly like raw SQL while enforcing absolute type safety at compile time.

---

## The Core Philosophy

1. **The DB is the Subject:** Every operation begins with the database instance (`db.select`, `db.insert`). Entities are pure, lightweight data structures without hidden state or active-record side effects.
2. **Visual Mapping, No Re-invention:** The DSL is designed to map directly to your mental model of SQL. `User::AGE.gt(18)` produces `age > 18` seamlessly. No complex object-graph abstractions.
3. **Zero Default Contagion:** Partial column scans shouldn't force your fields to implement `Default` or be wrapped in risky `Option` wrappers. `cherry` introduces precise compile-time projection mapping.
4. **Compile-Time Typestate Patterns:** Builders use Rust's type states to modify method availability and return types dynamically. An insert statement with a `returning` clause changes the signature of `.execute()` at compile time.

---

## 1. Defining Models & DTOs

Deriving `Model` automatically exposes your struct to the query engine and generates uppercase static column constants (e.g., `User::ID`, `User::NAME`) for compile-time safe DSL filtering.

```rust
use cherry::{Model, FromRow};

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

---

## 2. Querying (Select)

### Simple Queries & Method Idempotency

Chain selectors intuitively. Calling `.filter()` multiple times automatically combines them with `AND` expressions via internal AST nodes, or you can use native bitwise `&` and `|` operators to form explicit logical groups.

```rust
// Fetch all active adults named Alice or Bob
let users = db.select::<User>()
    .filter((User::NAME.eq("Alice") | User::NAME.eq("Bob")) & User::AGE.gte(18))
    .filter(User::IS_ACTIVE.eq(true)) // Automatically appended using AND under the hood
    .order_by(User::AGE.desc())
    .limit(10)
    .load()?; // Returns Vec<User>
```

### Advanced Projections (DTOs & Tuples)

Avoid loading massive rows into memory. Select specific columns and map them either into a lightweight DTO or directly into an anonymous Rust tuple.

*Ergonomics Note: If you only need a single column, pass it directly. For multiple columns, wrap them in a tuple.*

```rust
// Ergonomics: Single column requires NO tuple wrapping
let user_ids = db.select::<User>()
    .columns(User::ID) 
    .filter(User::AGE.gt(18))
    .load_as::<i64>()?; // Returns Vec<i64>

// Scenario A: Load multiple fields directly into a DTO struct
let profiles: Vec<UserProfileDTO> = db.select::<User>()
    .columns((User::ID, User::NAME, User::ROLE)) // Multiple columns wrapped in a tuple
    .filter(User::AGE.gt(30))
    .load_as()?;

// Scenario B: Load multiple fields into an anonymous tuple on the fly
let rows = db.select::<User>()
    .columns((User::ID, User::NAME))
    .filter(User::ROLE.eq("admin"))
    .load_as::<(i64, String)>()?; // Returns Vec<(i64, String)>. Auto-mapped via macro-implemented FromRow for Tuples
```

### Aggregate, Distinct & Grouping

```rust
// Perform quick scalar counting
let total_admins = db.select::<User>()
    .filter(User::ROLE.eq("admin"))
    .count()?; // Return `usize`

// Complex analytical grouping
let stats: Vec<RoleStatsDTO> = db.select::<User>()
    .distinct()
    .columns("role, count(1) as total") // Supports raw fragments when needed
    .group_by(User::ROLE)
    .having(User::AGE.gt(18))
    .load_as()?;
```

### Type-Safe JOINs (The Grail)

Multi-table joins leverage index-based tuple slicing under the hood. No schema-mapping nightmares, no field conflicts. If a `LEFT JOIN` fails to find a match, the `Option<M>` element decisively resolves to `None`.

```rust
// INNER JOIN returning paired strong entities
let rows = db.select::<User>()
    .inner_join::<Role>(User::ROLE.eq(Role::NAME))
    .filter(Role::PERMISSION_LEVEL.gt(5))
    .load_as::<(User, Role)>()?; // Return Vec<(User, Role)>

// LEFT JOIN capturing optional relationships safely
let results: Vec<(User, Option<Company>)> = db.select::<User>()
    .left_join::<Company>(User::COMPANY_ID.eq(Company::ID))
    .load_as()?; // Returns Vec<(User, Option<Company>)>. Returns None if first column of Company is NULL

```

---

## 3. Inserting (Single, Batch & Upsert)

### Unified Insertion & Auto-Chunking

Whether inserting a single reference, an array slice, or a large heap-allocated `Vec`, the engine accepts them through the identical `AsRows` trait boundary. If a batch contains more variables than SQLite's parameter limit (32,766 parameters), **`cherry` automatically and transparently splits it into multiple sequential queries under the hood.**

```rust
let new_user = User { 
    id: 0, 
    name: "Charlie".into(), 
    age: 25, 
    role: "user".into(), 
    is_active: true, 
    runtime_cache: vec![],
};
let batch_users = vec![user_a, user_b, user_c, ...]; // Massive dataset

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
    .on_conflict(User::ID)
    .update(User::ROLE.set("admin"))
    .execute()?;

// Advanced compound index ON CONFLICT DO UPDATE SET configuration
let affected = db.insert(&new_user)
    .on_conflict((User::NAME, User::ROLE)) // Composite unique key targets wrapped in a tuple
    .update(User::AGE.set(26))             // Standard overwrite variable binding
    .update(User::IS_ACTIVE.excluded())    // Generates `is_active = excluded.is_active` safely without binding variables
    .filter(User::AGE.lt(30))              // Context-aware AST clause aware of the upsert environment
    .execute()?;
```

### Typestate `.returning()` Clauses

By calling `.returning()`, the builder changes its compile-time type signature. The terminal `.execute()` method transitions from returning a `usize` (affected rows) to returning a strongly-typed `Vec` of requested elements.

```rust
// State Transition: Grab newly generated auto-increment IDs
let assigned_ids: Vec<i64> = db.insert(&batch_users)
    .returning(User::ID) // Single column
    .execute()?; // Returns Result<Vec<i64>> due to Typestate pattern

// State Transition: Grab multi-column tuple metrics on creation
let metadata: Vec<(i64, String)> = db.insert(&new_user)
    .returning((User::ID, User::ROLE)) // Multiple columns via tuple
    .execute()?; // Returns Result<Vec<(i64, String)>>
```

---

## 4. Updating & Deleting

`cherry` protects you from catastrophic accidents. Executing an update builder without calling `.set()` intercepts the execution pipeline to prevent invalid database transitions.

```rust
// Target fields and update securely
let updated_count: usize = db.update::<User>()
    .set(User::ROLE.set("moderator"))
    .filter(User::ID.eq(42))
    .execute()?;

// Delete with standard conditions
let deleted_count: usize = db.delete::<User>()
    .filter(User::IS_ACTIVE.eq(false))
    .execute()?;

```

Both actions natively support the unified `.returning()` and `.returning_as::<DTO>()` state pipelines exactly like the insertion architecture.

---

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

---

## 6. Connection Management & Pooling

`cherry` features a flexible `DatabaseBuilder` providing fine-grained optimization control. High-concurrency sqlite parameters can be configured fluidly out of the box.

### Single Connection Mode (Default)

Ideal for lightweight applications, command-line tools, or embedded environments.

```rust
use cherry::Database;
use cherry::database::builder::DatabaseBuilder;

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
use cherry::database::builder::DatabaseBuilder;

// Enable pooled variants fluidly (requires cargo "pool" feature)
let db = DatabaseBuilder::new("app.db")
    .with_default_pragmas() // Enables WAL mode, synchronous=NORMAL, busy_timeout=5000ms
    .pooled(10)             // Sets maximum connection pool size
    .build()?;

```

---

## 7. Transaction Processing

`cherry` abstracts the complexities of scope management via an automated closure context pipeline. If your closure block completes successfully with `Ok(())`, it auto-commits; if any error bubbles up via `?`, it triggers an automatic rollback.

### Shared CRUD Ergonomics

Because both `Database` and `Transaction` bind queries through a unified internal `Executor` design, **their public query APIs are 100% identical.** 

### Safe Compile-Time Borrowing (Single vs Pooled)

`cherry` leverages Rust's conditional feature flag mechanics to preserve absolute performance efficiency depending on your deployment choice:

* **Without `pool` feature:** `.transaction()` requires `&mut self` to guarantee exclusive single-thread isolation.
* **With `pool` feature:** `.transaction()` accepts `&self`. Spawning a transaction thread simply borrows a scoped node from the pool without locking down the parent database coordinator.

```rust
// Transaction management block via closure execution
db.transaction(|tx| {
    // Spawns query operations directly on the transaction context
    // The public API here is completely identical to calling db.insert / db.update
    let user_id = tx.insert(&new_user).returning(User::ID).execute()?[0];
    
    tx.update::<User>()
        .set(User::ROLE.set("verified"))
        .filter(User::ID.eq(user_id))
        .execute()?;
        
    Ok(()) // Automatically commits here, or rolls back if an Err is returned
})?;

```

---

## 8. Optional Cargo Features

`cherry` handles native primitives out-of-the-box (`bool`, `Vec<u8>`). Richer type mappings are isolated behind explicit optional features to maintain rapid compilation times and clean dependency trees.

| Feature Name | Target Crate | Database Representation | Defending Domain Realities |
| --- | --- | --- | --- |
| `uuid` | `uuid` | `BLOB (16 bytes)` | Enforces memory efficiency on disk, yet remains broad enough to leniency parse string fields on legacy reads. |
| `chrono` | `chrono` | `TEXT` | Formats to industry-standard ISO 8601 (RFC 3339) strings, allowing quick native database dictionary evaluations. |
| `time` | `time` | `TEXT` | Maps `time::OffsetDateTime` cleanly via structural RFC 3339 constraints. |
| `jiff` | `jiff` | `TEXT` | Integrates modern time mechanics (`jiff::Timestamp`) into string storage nodes. |
| `json` | `serde_json` | `TEXT` | Serializes flexible object notation configurations directly into standardized text elements. |
| `url` | `url` | `TEXT` | Maps strict validated web resource paths directly onto database nodes. |

Activate your ecosystem extensions inside your `Cargo.toml`:

```toml
[dependencies]
cherry = { version = "0.5.0", features = ["uuid", "chrono", "json", "pool"] }
```
