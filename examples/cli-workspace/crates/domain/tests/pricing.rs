use behave::prelude::*;

behave! {
    "pricing" {
        "other_package_case" {
            expect!(1 + 1).to_equal(2)?;
        }
    }
}
