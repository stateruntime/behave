//! Teardown blocks for cleanup that must run even when a test fails.
//!
//! `teardown` is panic-safe in sync tests: the body runs inside
//! `catch_unwind`, then the teardown code executes, then any panic is
//! re-raised. This guarantees cleanup for resources like temp files,
//! connections, or global state.

use behave::prelude::*;

/// Tracks how many connections are "open". In real code this could be a
/// database pool, a temp directory handle, or any resource that must be
/// released.
#[allow(dead_code)]
struct ConnectionPool {
    open: std::cell::Cell<u32>,
}

#[allow(dead_code)]
impl ConnectionPool {
    const fn new() -> Self {
        Self {
            open: std::cell::Cell::new(0),
        }
    }

    fn acquire(&self) -> u32 {
        let next = self.open.get() + 1;
        self.open.set(next);
        next
    }

    fn release(&self) {
        let current = self.open.get();
        if current > 0 {
            self.open.set(current - 1);
        }
    }

    fn active_count(&self) -> u32 {
        self.open.get()
    }
}

behave! {
    "teardown examples" {
        "basic cleanup" {
            setup {
                let pool = ConnectionPool::new();
                let _conn = pool.acquire();
            }

            teardown {
                pool.release();
            }

            "connection is open during the test" {
                expect!(pool.active_count()).to_equal(1)?;
            }
        }

        "teardown with setup variables" {
            setup {
                let mut log: Vec<&str> = Vec::new();
                log.push("setup");
            }

            teardown {
                log.push("teardown");
                // In a real test you would flush a file, close a socket, etc.
                let _ = &log;
            }

            "log records setup before test body" {
                log.push("test");
                expect!(log.len()).to_equal(2)?;
                expect!(log[0]).to_equal("setup")?;
            }
        }

        "nested teardowns run inner-first" {
            setup {
                let outer_resource = 42;
            }

            teardown {
                // Outer teardown runs second (like destructors).
                let _ = outer_resource;
            }

            "inner scope" {
                setup {
                    let inner_resource = outer_resource + 1;
                }

                teardown {
                    // Inner teardown runs first.
                    let _ = inner_resource;
                }

                "both resources available" {
                    expect!(outer_resource).to_equal(42)?;
                    expect!(inner_resource).to_equal(43)?;
                }
            }
        }
    }
}

#[allow(clippy::missing_const_for_fn, dead_code)]
fn main() {}
