template T() {
    var x = 0;

    if (x > 0) {
        component comp = T();
    }
}

component main = T();
