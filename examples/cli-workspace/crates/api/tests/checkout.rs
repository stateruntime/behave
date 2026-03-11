use behave::prelude::*;

behave! {
    "checkout" {
        "zeta_case" {
            expect!(2 + 2).to_equal(4)?;
        }

        focus "alpha_case" {
            expect!(true).to_be_true()?;
        }
    }
}
