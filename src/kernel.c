void kmain(void) {
    volatile unsigned short *vga_buffer = (unsigned short*)0xB8000;
    const char *msg = "Hello from C!";
    for (int i = 0; msg[i] != 0; i ++) {
        vga_buffer[i] = (0x0F << 8) | msg[i];
    }
    for(;;) {}
}
