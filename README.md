# Payment Engine

A simple payment engine that processes transactions (deposits, withdrawals, disputes, resolves, chargebacks) and manages client account balances.

## Architecture

The project follows a hexagonal (ports & adapters) architecture:

- **Domain** — core types: `Account`, `Amount` (fixed-point 4 decimals), `Transaction`
- **Ports** — repository traits: `AccountRepository`, `TransactionRepository`
- **Application** — use cases: `DepositUseCase`, `WithdrawalUseCase`, `DisputeUseCase`

Each use case is generic over its repository traits, making it testable with in-memory implementations and adaptable to any storage backend.

## Future Considerations

### Event Sourcing

The domain naturally maps to an event-sourced architecture — deposits, withdrawals, disputes, resolves, and chargebacks are all immutable events that modify account state. An event-sourced approach would:

- Store an append-only log of events instead of mutable account state
- Derive account balances by replaying events
- Provide a built-in audit trail
- Simplify dispute/resolve/chargeback logic (just reference prior events)

The current approach uses mutable account state with a separate transaction log, which is simpler but could be evolved into full event sourcing if audit or replay capabilities become important.
