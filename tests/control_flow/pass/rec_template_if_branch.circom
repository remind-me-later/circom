template T() {
    var x = 0;
    component comp;

    if (x > 0) {
        comp = T();
    }
}

component main = T();
