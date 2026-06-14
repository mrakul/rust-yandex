#include <stdio.h>

  
/* GCC аттрибуты для 32-бит x86 систем:

   - __attribute__((cdecl))   : вызывающая функция очищает стек (используется по умолчанию)

   - __attribute__((stdcall)) : вызываемая функция очищает стек

   - __attribute__((fastcall)): первые аргументы пердаются в регистрах, далее в стеке (MS style)

   для MSVC можно использовать напрямую: __cdecl, __stdcall, __fastcall.

*/


/* cdecl: стандартное соглашение о вызовах для C функций (вызывающая функция очищает стек) */
int __attribute__((cdecl)) add_cdecl(int a, int b) {
    return a + b;
}

  
/* stdcall: вызываемая функция очищает стек */
int __attribute__((stdcall)) add_stdcall(int a, int b) {
    return a + b;
}

  
/* fastcall: первые аргументы в регистрах (ECX, EDX) для множества компиляторов */
int __attribute__((fastcall)) add_fastcall(int a, int b) {
    return a + b;
}


/* variadic функция */
int __attribute__((cdecl)) sum_variadic(int count, ...) {

    /* Простой пример variadic функции  */

    #include <stdarg.h>

    va_list ap;
    va_start(ap, count);
    int s = 0;

    for (int i = 0; i < count; ++i)
        s += va_arg(ap, int);

    va_end(ap);
    return s;
}

  
int main(void) {

    int x = 10, y = 20;

    printf("add_cdecl:    %d\n", add_cdecl(x, y));
    printf("add_stdcall:  %d\n", add_stdcall(x, y));
    printf("add_fastcall: %d\n", add_fastcall(x, y));
    printf("sum_variadic: %d\n", sum_variadic(3, 1, 2, 3));

    return 0;

} 