template A(){
    signal input in1;
    signal output out;
    out <== in1;
}

component main {public [out]}= A();
