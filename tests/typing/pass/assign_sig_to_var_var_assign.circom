template T() {
    signal input a;
    // should pass, assigning var to signal,
    // with variable assignment operator
    var v = a; 
    // should pass, assign new numerical value to variable
    v = 1;
}

component main = T();
