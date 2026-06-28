// В VSCode на правую кнопку breakpoint'а => target == 30, разницы нет с C/C++ кодом, всё так же 

fn find_index(numbers: &[i32], target: i32) -> Option<usize> {
    for i in 0..7 {
        if numbers[i] == target {
            return Some(i);
        }
    }
    None
}

fn process_batch(numbers: Vec<i32>) {
    for i in 0..100 {
        if let Some(idx) = find_index(&numbers, i) {
            println!("Found {} at index {}", i, idx);
        }
    }
}

fn main() {
    let data = vec![10, 20, 30, 40, 50];
    process_batch(data);
}