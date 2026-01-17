cargo clippy --no-default-features -- -D clippy::all
cargo +nightly fmt --all

cd tests || printf 'ERROR: "tests" folder does not exist, cannot perform checks.'

cargo test --features cond_comp --no-default-features
cargo test --features cond_comp_beta --no-default-features
cargo test --features cond_comp_advanced --no-default-features
cargo test --features cond_comp_audio --no-default-features
cargo test --features cond_comp_experimental --no-default-features
cargo test --features cond_comp_graphics --no-default-features
cargo test --features cond_comp_networking --no-default-features
cargo test --features cond_comp_audio --no-default-features
cargo test --all-features