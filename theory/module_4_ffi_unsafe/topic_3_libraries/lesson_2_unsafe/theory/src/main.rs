// Как же правильно писать unsafe-код? Здесь есть несколько вариантов.
// 1. Путь первый: unsafe-функция
// Если функция может принимать данные, которые могут быть невалидными и вызывать undefined behavior, 
// она должна быть помечена ключевым словом unsafe перед fn. 
// Так вы скажете пользователю функции, что при её использовании нужно дополнительное внимание, а компилятору — что здесь есть небезопасные операции и поэтому такая функция может быть вызвана только в блоке unsafe. Теперь интерфейс выглядит так:

// Чтобы интерфейс был самодостаточным, а не требовал чтения исходников, в Rust принято в документации к unsafe‑функции явно описывать, чем именно она небезопасна и какие инварианты должен гарантировать вызывающий код, чтобы избежать неопределённого поведения. Для нашего примера это можно оформить так:  

/// Returns mutable references to many indices at once
/// # Safety
/// * indices do not overlap
/// * indices are not out of bound of entities array
unsafe fn get_entities_at<T, const N: usize>(entities: &mut [T], indices: [usize; N]) -> [&mut T; N] {
    // ...
} 

// Теперь пользователь функции будет знать, какие инварианты необходимо соблюсти, чтобы код оставался безопасным. Однако программисты не идеальны и могут случайно нарушить эти инварианты. Поэтому хорошей практикой является проверка соблюдения условий в debug-сборках — для этого используется макрос debug_assert.
// Напишем функцию проверки:


fn check_indicies_valid<const N: usize>(indices: &[usize; N], len: usize) -> bool {
    for index in indices {
        // out of bound
        if *index >= len {
            return false;
        }
        // index overlap
        if indices.iter().filter(|i| *i == index).count() > 1 {
            return false;
        }
    }
    true
} 

// И добавим функцию:

/// Returns mutable references to many indices at once
/// # Safety
/// * indices do not overlap
/// * indices are not out of bound of entities array
unsafe fn get_entities_at<T, const N: usize>(entities: &mut [T], indices: [usize; N]) -> [&mut T; N] {
    unsafe {
        // Check that indicies do not overlap and are not out of bound
        debug_assert!(check_indicies_valid(&indices, entities.len()));
        // преобразование референса в указатель, чтобы можно было вызвать
        // get_unchecked_mut в цикле
        let entities: *mut [T] = entities;
        // просто аллокация памяти на стеке с неинициализироваными значениями
        // Небольшая оптимизация, так как лишний раз не записываем нули,
        // которые потом все равно будут перезаписаны
        let mut output: [&mut T; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            let index: usize = *indices.get_unchecked(i);
            output[i] = (&mut *entities).get_unchecked_mut(index);
        }

        output
    }
} 

// Как пример похожей логики из стандартной библиотеки можно привести NonNull::new_unchecked (пример упрощён, оставлены только нужные для примера строки):

/// Creates a new `NonNull`.
/// # Safety
/// `ptr` must be non-null.
pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
    unsafe {
        assert_unsafe_precondition!(
            check_language_ub,
            "NonNull::new_unchecked requires that the pointer is non-null",
            (ptr: *mut () = ptr as *mut ()) => !ptr.is_null()
        );
        NonNull { pointer: ptr as _ }
    }
} 

// 2. Путь второй: оборачивание в безопасную обёртку
// unsafe как эффект не должен распространяться бесконечно вверх по графу вызовов, иначе весь код стал бы unsafe. Поэтому в какой-то момент эти инварианты должны гарантироваться. Обернём наш код в безопасную функцию, гарантировав соблюдение инвариантов: 

fn get_entities_at<T: Visibility, const N: usize>(entities: &mut [T], indices: [usize; N]) -> Result<[&mut T; N], ()> {
    if !check_indicies_valid(&indices, entities.len()) {
        return Err(());
    }
    
    let mut entities: Vec<_> = entities.into_iter().filter(|entity| entity.is_visible()).collect();
    let entities: &mut [&mut T] = entities.as_mut();

    // Только секция unsafe
    let output = unsafe {
        debug_assert!(check_indicies_valid(&indices, entities.len()));
        // преобразование референса в указатель, чтобы можно было вызвать
        // get_unchecked_mut в цикле
        let entities: *mut [&mut T] = entities;
        // просто аллокация памяти на стеке с неинициализироваными значениями
        // Небольшая оптимизация, так как лишний раз не записываем нули,
        // которые потом всё равно будут перезаписаны
        let mut output: [&mut T; N] = MaybeUninit::uninit().assume_init();
        for i in 0..N {
            let index: usize = *indices.get_unchecked(i);
            output[i] = (&mut *entities).get_unchecked_mut(index);
        }

        output
    };

    Ok(output)
} 

// Но останется ли такой код безопасным? Читающему код придётся снова разбираться, какие инварианты необходимо соблюсти, чтобы код оставался безопасным. Этого можно избежать, если явно обозначать, какие инварианты соблюдены. По аналогии с комментариями Safety у функций, для unsafe-блоков принято обозначать соблюдение инвариантов ключевым словом SAFETY:  

fn get_entities_at<T: Visibility, const N: usize>(entities: &mut [T], indices: [usize; N]) -> Result<[&mut T; N], ()> {
    if !check_indicies_valid(&indices, entities.len()) {
        return Err(());
    }
    
    let mut entities: Vec<_> = entities.into_iter().filter(|entity| entity.is_visible()).collect();
    let entities: &mut [&mut T] = entities.as_mut();

    // SAFETY: the code is safe because we just checked that
    // indicies do not overlap and are not out of bound
    let output = unsafe {
        // тот же код, что и был
    };

    Ok(output)
} 

// Теперь при изменении кода можно просто перепроверить соблюдение инвариантов. Но это не единственное преимущество. При написании такого комментария программист лишний раз обдумывает, какие инварианты необходимо соблюсти. Кроме того, это упрощает поиск ошибок: можно сравнить условия, при которых возникает баг, с теми, что прописаны в комментариях. При этом ошибки безопасности будут возникать только в unsafe-блоках. 
// Правда, при проявлении бага в одном месте его причиной может оказаться unsafe-блок в совершенно другом месте. Поэтому желательно сузить область поиска — старайтесь минимизировать размер unsafe-блоков.  


// Примеры безопасных обёрток
// Буфер из C:

pub fn copy_from_c_buffer(src: *const u8, len: usize) -> Result<Vec<u8>, &'static str> {
    if src.is_null() {
        return Err("null pointer");
    }
    // SAFETY: вызывающий гарантирует валидность [src, src+len) на время вызова.
    let slice = unsafe { std::slice::from_raw_parts(src, len) };
    Ok(slice.to_vec())
} 

// C-строка:

use core::ffi::{c_char, CStr};

pub fn c_str_to_str<'a>(ptr: *const c_char) -> Result<&'a str, &'static str> {
    if ptr.is_null() {
        return Err("null pointer");
    }
    let c = unsafe { CStr::from_ptr(ptr) };
    c.to_str().map_err(|_| "invalid utf-8")
} 

// NonNull:

use core::ptr::NonNull;

pub fn bump(counter: *mut u32) -> Result<(), &'static str> {
    let Some(p) = NonNull::new(counter) else {
        return Err("null pointer");
    };
    // SAFETY: единственный доступ через raw pointer.
    unsafe { *p.as_ptr() = p.as_ptr().read().wrapping_add(1) };
    Ok(())
} 

// Инженерная цель здесь — спрятать unsafe внутрь маленьких функций, наружу отдавать Result и типы с инвариантами.

// 3. Путь третий: unsafe, которого нет
// Лучшее, что можно сделать с unsafe, это не писать его. Если ту же логику можно написать в safe rust, то пишите её в safe rust 
// (разве что можно как небольшое исключение сказать про случай, когда с unsafe мы можем получить хороший прирост производительности в критическом для производительности месте). 
// Стоит сто раз подумать, прежде чем писать unsafe-код.

// (Проектирование безопасного интерфейса)
// Лучшее, что можно сделать с unsafe, это не писать его. 
// Если ту же логику можно написать в safe rust, то пишите её в safe rust. 
// Конечно, исключением является ситуация, когда с unsafe мы можем получить хороший прирост производительности в критическом для производительности месте, но и здесь стоит сто раз подумать, прежде чем писать unsafe-код.



// Quiz:
// Что из перечисленного в этом фрагменте приводит к UB? Выберите верный вариант.

let mut five = 5;
let x: *mut i32 = &mut five;
unsafe {
    *x = 10;  // запись через сырой указатель
    println!("{}", *x);
} 

// Правильный ответ
// Всё правильно, UB не будет
// Одна &mut используется для создания *mut, дальше доступ идёт через сырой указатель в unsafe. Код компилируется и выполняется без неопределённого поведения.

// 3. What guarantees that the pointer is not dangling?
// The rules of variable lifetimes (scopes).
// In Rust, a local variable like five is allocated on the stack.
// Its lifetime is strictly tied to the scope (the curly braces { ... }) in which it is declared.

// Because the unsafe block is inside the same scope where five is declared, 
// the compiler guarantees that five is alive and its memory is valid at the exact moment *x = 10; is executed.
// The pointer x is just holding a memory address. 
// The guarantee that this address is valid comes from the fact that the variable living at that address (five) has not been dropped yet.

// Summary
// Why unsafe? Because the compiler cannot prove raw pointers are safe. It forces you to promise that you checked.
// Is it UB? In this specific snippet, no. But raw pointers can easily cause UB if you misuse them (like using them after the variable is dropped).
// Why not dangling? Because five is on the stack and lives until the end of its scope. The unsafe block executes before that scope ends, so the memory is guaranteed to be valid.