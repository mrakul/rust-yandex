1. ldd target/release/dyn_lib_linking:
(до внесения изменений)
>
    linux-vdso.so.1 (0x0000755b05b68000)
    libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x0000755b05ace000)
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x0000755b05800000)
    /lib64/ld-linux-x86-64.so.2 (0x0000755b05b6a000)c
>


2. ldd target/release/dyn_lib_linking:
(после внесения изменения, libz подключился, естественно, после добавления вызова)
    linux-vdso.so.1 (0x00007bb85010f000)
    libz.so.1 => /lib/x86_64-linux-gnu/libz.so.1 (0x00007bb850087000)
    libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1 (0x00007bb850059000)
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007bb84fe00000)
    /lib64/ld-linux-x86-64.so.2 (0x00007bb850111000)
