# Useful command automations to run in this repository.
# You must install the "just" command using "cargo install just" in order to use this file.
# In order to do that, you must install the Rust toolchain. See https://rustup.rs for details.

# Install the global dependencies required to run the automations in this file.
deps:
    rustup component add llvm-tools-preview
    cargo install cargo-llvm-cov http-server

# Remove generated files from the repository.
clean:
    cargo clean

# Use static analysis to check the code.
lint:
    cargo-clippy

# Reformat the code according to standard Rust formatting rules.
format:
    cargo-fmt

# Check that the code is formatted according to standard Rust formatting rules.
format-check:
    cargo-fmt --check

# Run the unit and integration tests.
test:
    cargo test

# Run the unit and integration tests, and print a code coverage report to the terminal.
test-coverage:
    rm -rf target/llvm-cov target/llvm-cov-target
    cargo llvm-cov --branch

# Run the unit and integration tests, and produce an HTML code coverage report.
test-coverage-html:
    rm -rf target/llvm-cov target/llvm-cov-target
    cargo llvm-cov --branch --html

# Run the unit and integration tests, and use an HTTP server to publish the HTML code coverage report.
test-coverage-html-serve: test-coverage-html
    http-server --index target/llvm-cov/html

# Generate the rustdoc documentation for the crate.
doc:
    RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --workspace --no-deps

# Generate the rustdoc documentation for the crate and use an HTTP sever to publish it.
doc-serve: doc
    http-server --index target/doc
