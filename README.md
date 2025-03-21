# Sol-Ez
Sol-Ez is a Solana contract framework inspired by gRPC/protobuf, designed to generate template code and simplify the process of writing Solana programs.

## Why Sol-Ez?
- Reduces Boilerplate: Automatically generates necessary Rust structures.
- Improves Readability: Enforces a structured definition of contracts.
- Simplifies Development: Manage accounts with a simple API.

## Getting Started
- Define your accounts and instructions using the Sol-Ez syntax.
- Use the Sol-Ez code generator to produce the Rust contract.
- Implement your business logic by defining a contract struct and implementing the generated trait.

## Example

### Define a Contract
```
account Count {
    data: u64 = 1;
}

accounts Initialize {
    init counter: Count = 1;
    payer: Payer = 2;
    program: Program = 3;
}

accounts Update {
    mut counter: Count = 1;
}

contract Counter {
    instruction initialize(Initialize) = 1;
    instruction update(Update) = 2;
}
```

### Add build step
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    sol_ez_gen::generate("contracts/contract.ez", "src/contract.rs")?;
    Ok(())
}
```

### Implement contract 
```rust

mod contract;

type EFN = for<'a, 'b, 'info> fn(
    &'a Pubkey,
    &'info [AccountInfo<'info>],
    &'b [u8],
) -> Result<(), ProgramError>;

pub const FN: EFN = CounterDispatcher::<Counter>::dispatch;

pub struct Counter;

impl CounterContract for Counter {
    fn initialize(ctx: Context<contract::Initialize>) -> ProgramResult {
        let payer = ctx.accounts.payer;
        let sys_program = ctx.accounts.program;
        let owner = ctx.program_id;

        let account = Count { data: 0 };
        let counter = ctx
            .accounts
            .counter
            .init(account, &payer, &sys_program, owner)?;

        msg!("Counter initialized with value: {}", counter.as_ref().data);
        
        Ok(())
    }

    fn update(mut ctx: Context<contract::Update>) -> ProgramResult {
        ctx.accounts.counter.as_ref_mut().data += 1;
        let counter = ctx.accounts.counter.apply()?;
        
        msg!("Counter incremented to: {}", counter.as_ref().count);
        
        Ok(())
    }
}
```

## License

Sol-Ez is licensed under [Apache 2.0](./LICENSE).