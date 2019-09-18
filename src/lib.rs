pub mod schema;
pub mod models;

#[macro_use]
extern crate diesel;

/*
? which productions are, or are not, idempotent?
 --> creating entities, peers, or events is non-idempotent
? calculate record by record, or relationally?
 --> relationally, so it's easy to rev the schema by blowing it away and recreating it

- incoming event
    - local: new send message event (body)
    - remote: serialized send message event (body)
- create entry in events table
    - blow up if it already exists
- until no new data is created,
    - apply every rule in system
        - insert every generated row not already present in database
        - return true if any data was generated

refinements:
- apply rules in topographic order
*/

