MEMORY
{
  /* Leave 16k for the default bootloader on the Feather M4 */
  FLASH (rx)   : ORIGIN = 0x00000000 + 16K, LENGTH = 512K - 16K
  /* Usually 192K RAM. Reserve 1K at the end for storing panic dumps */
  RAM (xrw)    : ORIGIN = 0x20000000, LENGTH = 191K
  PANDUMP (rw) : ORIGIN = 0x20000000 + 191K, LENGTH = 1K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_panic_dump_start = ORIGIN(PANDUMP);
_panic_dump_end   = ORIGIN(PANDUMP) + LENGTH(PANDUMP);

