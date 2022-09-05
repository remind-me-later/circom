// despite missing this semicolon
pragma circom 2.0.6

// shows error about invalid include
include file.txt;

template T() {
    var a = 2;
}

component main = T();
