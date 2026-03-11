# Payment Engine

A CLI tool that processes financial transactions from a CSV file and outputs client account balances.

```
cargo run -- transactions.csv > accounts.csv
```

## Usage

Input CSV format:

```csv
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
withdrawal, 1, 3, 1.5
dispute, 1, 1,
resolve, 1, 1,
chargeback, 2, 2,
```

Output CSV format:

```csv
client,available,held,total,locked
1,1.50,0.00,1.50,false
2,0.00,0.00,0.00,true
```

Whitespace in the input CSV is tolerated. Malformed rows are silently skipped. Duplicate transaction IDs are rejected.

A chargeback freezes the client's account. Once frozen, the account rejects all subsequent transactions (deposits, withdrawals, disputes, resolves, and chargebacks). Rejected transactions are reported to stderr.

## Architecture

Hexagonal (ports & adapters) architecture with three layers:

```
CSV file ──► CsvTransactionReader (Iterator) ──► TransactionManager ──► Use Cases ──► Repos
                                                                                       │
stdout   ◄── CsvAccountWriter ◄── AccountRepository.all() ◄───────────────────────────┘
```

### Domain

Core types with no external dependencies:

- **`Amount`** — fixed-point arithmetic with 4 decimal places, stored as `i64` internally. Parsed from strings via `FromStr`, displayed as `X.XX`. Supports `Add`, `Sub`, `is_negative()`.
- **`Account`** — client balance with `available`, `held`, `locked` fields. `total() = available + held`.
- **`Transaction`** — a `TransactionType` enum (Deposit, Withdrawal, Dispute, Resolve, Chargeback) plus `client: u16`, `tx: u32`, `amount: Option<Amount>`.

### Ports

Trait-based boundaries that decouple application logic from infrastructure:

- **`AccountRepository`** — `find_by_client_id`, `save`, `all`
- **`TransactionRepository`** — `find_by_tx_id`, `save`
- **`DisputeRepository`** — `is_disputed`, `mark_disputed`, `remove_dispute`
- **`AccountWriter`** — `write_all(&[Account])` for output
- Use case traits: `Deposit`, `Withdraw`, `DisputeTx`, `Resolve`, `Chargeback`

All repository and use case traits are automocked via `mockall` for unit testing.

### Application

- **Use cases** — one struct per transaction type (`DepositUseCase`, `WithdrawalUseCase`, `DisputeUseCase`, `ResolveUseCase`, `ChargebackUseCase`), each generic over repository traits.
- **`TransactionManager`** — routes `Transaction` to the correct use case based on `TransactionType`.

### Adapters

- **`InMemoryAccountRepo`**, **`InMemoryTransactionRepo`**, **`InMemoryDisputeRepo`** — `HashMap`-based in-memory storage. All implement their traits for both direct use and `Rc<RefCell<T>>` (shared ownership across use cases).
- **`CsvTransactionReader`** — implements `Iterator<Item = Transaction>`, streaming one transaction at a time from a CSV source via the `csv` crate with serde deserialization. Skips malformed rows.
- **`CsvAccountWriter`** — implements `AccountWriter`, serializing accounts to CSV via the `csv` crate.

## Design Decisions

### Fixed-Point Arithmetic

Amounts use `i64` with 4 implicit decimal places (e.g., `15000` = `1.5000`) instead of floating-point. This avoids rounding errors in financial calculations. Parsed from strings, never from floats.

### Streaming CSV Input

The CSV reader implements `Iterator` rather than buffering all transactions into a `Vec`. This processes transactions one at a time with constant memory overhead regardless of file size.

### Shared Repos via `Rc<RefCell<T>>`

All use cases need the same repository instances. Repository traits are implemented for `Rc<RefCell<T>>` so repos can be shared across use cases while `main` retains access for final output.

### Generic Use Cases (No `dyn`)

Use cases are generic over repository traits rather than using `Box<dyn Trait>`. This gives zero-cost abstraction with static dispatch while keeping use cases testable with mocks.

### Automocked Traits

Both repository traits and use case traits use `#[cfg_attr(test, mockall::automock)]`. Use case tests mock repositories, and `TransactionManager` tests mock use cases. No test-only accessor methods needed.

## Testing

```
cargo test
```

67 tests total:
- Unit tests for each use case (deposit, withdrawal, dispute, resolve, chargeback)
- Unit tests for `TransactionManager` routing
- Unit tests for CSV reader (parsing, whitespace, malformed rows, streaming)
- Unit tests for CSV writer
- Unit tests for in-memory repos (including `Rc<RefCell<>>` impls)
- E2E tests running the binary with fixture CSVs (basic, frozen account, duplicate tx)

## Future Considerations

### Event Sourcing

The domain naturally maps to an event-sourced architecture — deposits, withdrawals, disputes, resolves, and chargebacks are all immutable events that modify account state. An event-sourced approach would:

- Store an append-only log of events instead of mutable account state
- Derive account balances by replaying events
- Provide a built-in audit trail

### Swappable I/O Adapters

The CSV reader and writer can be replaced with other adapters (e.g., HTTP, database) since use cases depend on traits, not concrete implementations.

### Why Not Async (Tokio)

The engine is intentionally synchronous. The only I/O is reading a local CSV file and writing to stdout — there are no network calls, database connections, or concurrent I/O sources where `async` would help. All data lives in `HashMap`/`HashSet` behind `Rc<RefCell<T>>`, which isn't `Send`/`Sync` — adopting Tokio would require migrating to `Arc<Mutex<T>>` and making all traits async for no throughput benefit. Transaction ordering also matters (disputes reference earlier deposits), so parallelism would add complexity without correctness guarantees. If the engine later grew an HTTP API or a real database backend, async would become justified.

## AI Disclosure

This project was developed with the assistance of **Claude Code** (Anthropic's CLI tool). AI was used as a pair-programming partner throughout development for:

- **Architecture discussions** — evaluating hexagonal architecture trade-offs, `Rc<RefCell<T>>` vs `Arc<Mutex<T>>`, static vs dynamic dispatch
- **TDD workflow** — writing failing tests first, then implementing code to make them pass
- **Code generation** — scaffolding use cases, error types, trait implementations, and test fixtures
- **Design decisions** — fixed-point arithmetic approach, streaming vs buffering, async vs sync trade-offs
- **README authoring** — structuring documentation and articulating design rationale

All architectural decisions, domain modeling choices, and error handling strategies were directed and validated by the author. The AI acted as an accelerator — not a decision-maker.
