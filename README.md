# memwatch

Really quick small program to run a child program and send control+c if your free memory goes below a threshold.

To compile: `cargo build --release`, to run `target/release/memwatch command`. `-g NUM` is an optional arg that changes the threshold (in gigs). By default it's 1 gig.
