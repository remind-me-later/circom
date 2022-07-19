#![cfg(test)]

use circom_error::{error_code::ReportCode};

use crate::{parser_logic};

#[test]
pub fn basic_comments() {
    let src = r###"
    // single line
    /* multiline in a line */  
    /* 
     * multiline spanning multiple lines
     */
   "###;

    parser_logic::parse_file(src, 0).unwrap();
}

#[test]
pub fn unclosed_multiline_comment() {
    let src = r###"
    /* Oopsie...  
   "###;

    let res = parser_logic::parse_file(src, 0).err().unwrap();

    assert_eq!(*res.get_code(), ReportCode::ParseFail);
}

#[test]
pub fn multiplier2() {
    let src = r###"
    pragma circom 2.0.0;

    /*This circuit template checks that c is the multiplication of a and b.*/  

    template Multiplier2 () {
        // Declaration of signals.  
        signal input a;  
        signal input b;  
        signal output c;  

        // Constraints.  
        c <== a * b;
    }
    "###;

    parser_logic::parse_file(src, 0).unwrap();
}
