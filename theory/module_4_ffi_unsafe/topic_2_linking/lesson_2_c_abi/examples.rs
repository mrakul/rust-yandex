// Для примера рассмотрим заполнение структуры jvmtiEventCallbacks из 
// jvmti. В примере указана неполная версия, поэтому она неприменима к настоящему jvmti, зато хорошо демонстрирует использование Option для указателей на функции. 
// Структура jvmtiEventCallbacks хранит в себе колбэки (функции) для подписки на разные события jvm. Она имеет примерно следующий вид:

typedef struct {
                              /*   52 : Thread Start */
    jvmtiEventThreadStart ThreadStart;
                              /*   54 : Class File Load Hook */
    jvmtiEventClassFileLoadHook ClassFileLoadHook;
                              /*   65 : Method Entry */
    jvmtiEventMethodEntry MethodEntry;
                              /*   67 : Native Method Bind */
    jvmtiEventNativeMethodBind NativeMethodBind;
} jvmtiEventCallbacks; 

// А вот её реализация в Rust:

#[derive(Default)]
#[repr(C)]
pub struct jvmtiEventCallbacks {
    pub ThreadStart: Option<jvmtiEventThreadStart>,
    pub ClassFileLoadHook: Option<jvmtiEventClassFileLoadHook>,
    pub MethodEntry: Option<jvmtiEventMethodEntry>,
    pub NativeMethodBind: Option<jvmtiEventNativeMethodBind>,
} 

// Поля структуры расположены в том же порядке, что и в оригинальной структуре. Это важно для совместимости с С, так как доступ к полям на низком уровне осуществляется через смещения от начала структуры.
// Можно написать колбэк на загрузку класса. Для этого нужно реализовать jvmtiEventClassFileLoadHook. Сигнатура в С выглядит следующим образом:

typedef void (JNICALL *jvmtiEventClassFileLoadHook)
    (jvmtiEnv *jvmti_env,
     JNIEnv* jni_env,
     jclass class_being_redefined,
     jobject loader,
     const char* name,
     jobject protection_domain,
     jint class_data_len,
     const unsigned char* class_data,
     jint* new_class_data_len,
     unsigned char** new_class_data); 

// Она же в Rust — в следующей теме вы детально разберёте создание такого кода:

pub type jvmtiEventClassFileLoadHook = unsafe extern "C" fn(
    jvmti_env: *mut jvmtiEnv,
    jni_env: *mut JNIEnv,
    class_being_redefined: jclass,
    loader: jobject,
    name: *const c_char,
    protection_domain: jobject,
    class_data_len: jint,
    class_data: *const c_uchar,
    new_class_data_len: *mut jint,
    new_class_data: *mut *mut c_uint,
); 

// Эта функция вызывается JVM. JVM при вызове передаёт название загружаемого класса, а также его размер и остальные параметры. А хук просто выводит название загружаемого класса и его размер:

// hooks.rs
pub(super) unsafe extern "C" fn classfile_load_hook(
    _jvmti_env: *mut jvmtiEnv,
    _jni_env: *mut jni_sys::JNIEnv,
    _class_being_redefined: jclass,
    _loader: jobject,
    name: *const c_char,
    _protection_domain: jobject,
    class_data_len: jint,
    _class_data: *const c_uchar,
    _new_class_data_len: *mut jint,
    _new_class_data: *mut *mut c_uint,
) {
    let name = unsafe { std::ffi::CStr::from_ptr(name) };
    debug!("loading class: {name:?} with size {class_data_len}");
} 

// Эту функцию можно передать в структуру jvmtiEventCallbacks, содержащую колбэки. В примере ниже структура заполняется тремя функциями:

    let callbacks = jvmtiEventCallbacks {
        NativeMethodBind: Some(hooks::on_native_method_bind),
        ClassFileLoadHook: Some(hooks::classfile_load_hook),
        MethodEntry: Some(hooks::handle_method_entry),
        ..Default::default()
    }; 

// Вызов ..Default::default() здесь заполняет все остальные поля Default — в данном примере это 
// None, так как использовался 
// Derive(Default). Поля в итоге компилируются в виде нулевых указателей.
// Заучивать все детали и варианты кода не нужно, но будет полезно разобрать несколько примеров, так как в будущих уроках вы будете связывать Rust и C. extern "C" в Rust автоматически учитывает ABI‑выравнивания, о которых идёт речь.
// Простейший пример, без пэддинга:

typedef struct {
    uint16_t first_one;   // 2 байта
    uint16_t second_one;  // 2 байта
} sample_struct_without_padding; /* sizeof = 4, align = 2 */ 

// В Rust эта структура бы выглядела следующим образом:

#[repr(C)]
struct SampleStructWithoutPadding {
  first_one: u16,
  second_one: u16
} 

// repr(c) обеспечивает гарантию, что структура будет соответствовать её аналогу в C, обеспечивая совместимость с С кодом, если использовать её между языками.
// Пример с пэддингом:

typedef struct {
    sample_struct_without_padding inner; /* байты по смещениям: [0..3] */
    uint8_t other;                       /* байт по смещению: [4] */
    /* byte [5] = pad */
} OuterA;
/* Эквивалентно (явно с пэддингом):
typedef struct {
    uint16_t first_one;   // [0..1]
    uint16_t second_one;  // [2..3]
    uint8_t other;        // [4]
    uint8_t /* pad */     // [5]
} OuterA_explicit;
*/ 

Эквивалент на Rust:

#[repr(C)]
struct OuterA {
  inner: sample_struct_without_padding,
  other: u8
} 