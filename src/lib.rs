pub mod schema;
pub mod models;

#[macro_use]
extern crate diesel;



/*
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

