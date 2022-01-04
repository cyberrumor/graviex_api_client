use std::collections::HashMap;
use std::error::Error;

extern crate serde;
// use serde::{Serialize, Deserialize};

mod lib;

#[allow(clippy::never_loop)]
fn main() {
    // library tests
    // run_tests();
    loop {
	// your algo here
	break
	//

    }
    println!("The main loop was broken, program has exited.");
}


#[allow(unused)]
fn run_tests() {
    lib::test_no_params();
    lib::test_params();
    // lib::mutable_state_tests();
    println!("tests complete");
}

