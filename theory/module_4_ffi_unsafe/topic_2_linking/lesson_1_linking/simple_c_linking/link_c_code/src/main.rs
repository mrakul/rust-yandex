unsafe extern "C" {                       // unsafe поскольку компилятор не даёт гарантии на код из других языков
    fn add(a: i32, b: i32) -> i32; // объявление вызываемой функции
}

fn main() {
    let a = 5;
    let b = 7;
    
    let sum = unsafe { add(a, b) }; 
    println!("{} + {} = {}", a, b, sum);
} 