template T() {
    signal input a;
    signal input b;
    // should pass, assigning var to signal,
    // with variable assignment operator
    var c[2] = [a, b]; 
    // should pass, assign new numerical value to variable
    c = [1, 2];
}

component main = T();
