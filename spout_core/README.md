# Identity 
- Iroh PublicKey + Profile

Public keys can have multiple profiles

# Profile
- Id
- Description
- Picture

Heart of spout social. Profiles are attached to Groups, group users, bots, whatever. Its intended that People (aka - humans), will have multiple profiles.

## Structure

### `./models/` 

Houses all the database related code. Migrations, database types, and the actual data access modules go here

### `./service`

Modules here define two things

1. The actual "Services" which wrap up the biz logic to interface with models
2. The server implementations which will be exposed to other peers


It might seem like there is a lot of "proxy" code given the current state of things. Long term I think establishing this seperation of concerns will pay off. Medium term.. Testing :)

## Dev Notes / Todos

### TODO
- Need to actually tag errors as infra or application. We should be able to just impl `impl From<WhateverError> for ResourceError` on the upstream types and just convert them downstream.

### Notes

- I'm trying using the `sqlx::Any`Db,Conn,whatever so in the future we can support different DBs. But, the actual sql dialects won't magically be compatable. For the forseeable future SQLite will be more than enough

