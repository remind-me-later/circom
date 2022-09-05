template T() {
    var b;
    var a = 1;

    if (a == 2) {
        a = 3;
    } else {
        a = 4;
    }
}

component main = T();
