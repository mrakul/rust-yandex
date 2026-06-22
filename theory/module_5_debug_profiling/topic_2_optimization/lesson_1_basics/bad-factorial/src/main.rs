fn factorial(n: u64) -> u64 {
    // Видимо, в задании предполагается, что можно увеличить до u128 разрядность
    // let mut result = 1;
    let mut result: u128 = 1;
    for i in 1..=n {
        result *= i;
    }
    result
}

fn main() {
    println!("{}", factorial(100));
} 

/*
    (gdb) break 4
    Breakpoint 2 at 0x55555556886a: file src/main.rs, line 4.
...
    continue
    Breakpoint 2, bad_factorial::factorial (n=20) at src/main.rs:4
    4               result *= i;
    (gdb) p i
    $26 = 12
    (gdb) p result
    $27 = 39916800
    (gdb) continue
    Continuing.

    Breakpoint 2, bad_factorial::factorial (n=20) at src/main.rs:4
    4               result *= i;
    (gdb)  
    ...
    (gdb) continue
    Continuing.
    Breakpoint 2, bad_factorial::factorial (n=20) at src/main.rs:4
    4               result *= i;
    (gdb) p i
    $30 = 15
    (gdb) p result
    $31 = 87178291200
    ...
    $3 = 20
    (gdb) p result
    $4 = 121645100408832000
    (gdb) c
    Continuing.

    Breakpoint 1, bad_factorial::factorial (n=100) at src/main.rs:4
    4               result *= i;
    (gdb) p i
    (!) $5 = 21
    (gdb) p result
    $6 = 2432902008176640000
    (gdb) c
    Continuing.

*/