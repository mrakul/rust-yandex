use broken_app::concurrency::race_increment;
use broken_app::use_after_free;

// Добавил два теста по сравнению с исходным broken-app (refernce-app содержал тест с гонкой):
// - race_increment_is_correct() - многопоточность, гонка при инкременте
// - test_use_after_free

#[test]
fn race_increment_is_correct() {
    let total = race_increment(1_000, 4);
    assert_eq!(total, 4_000);
}

const ULTIMATE_ANSWER_OF_LIFE: i32 = 42;

#[test]
fn test_use_after_free() {
    unsafe {
        assert_eq!(use_after_free(), ULTIMATE_ANSWER_OF_LIFE + ULTIMATE_ANSWER_OF_LIFE);
    }
}


