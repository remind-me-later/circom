template T() {
    signal input a;
    var c;
    // should fail, assigning var to signal,
    // with signal assignment operator
    c <== a; 
}

component main = T();
