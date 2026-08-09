# Why this fork exists

Since original compiler is mostly abbandoned
and I want for b lang to potentially became a *universal assembly* for real
I decided to make this fork.

It should follow spirit of original one
but it introduces some changes in favor of historic mode and code portability.

Things that concerned me most and that I decided to change were:
- Char constants were 2 bytes on each platform, and they had different API on different platforms
  So I made them to match size of platforms word
- Moved number and char parsing to codegens, for them to validate
  and store in correct order
- Each constant stored in memory now read to be in a BE form
  as it was written
  so libb functions for BE/LE 16 bit targets act the same
  Other targets should also
- Using char and lchar should act the same for char constants and for strings
  So I turned strings into real b strings with byte packing
  And char and lchar now search for first non zero byte to get a character
  Also strings now end with '`*e`'
  so you could treat non full word vectors as strings, if they end with '`*e`'
  Example: A 'W', 'O', 'R', 'D', 0x42, '`*e`';
  So you could replace parts of those vector strings if needed,
  even with utf-8
- Added `*e` control character that is equal 4
  because in bref it says that it should be equal to EOT
- Removed data section from IR and just left Globals and Strings
  so each codegen could interpret it in it's own way
  but mostly just transform them to read correct by char/lchar
- Single error now don't stop compilation, but tries to show most of them
  until hits limit
  So it acts for codegens and lexer now same as for compiler errors
- Since API is now universal for stuff in memory, implemented most of libb in B
  and not in asm for each target
- most of -hist stuff should work in "modern" version
  to have universal libb in hist subset
- added -hist flag to btest, so now we could filter out some tests for it
  and test specific stuff for it

Probably some of those features were planned in original b compiler,
but will likely never be implemented,
because this is not an interesting content or just a waste of time.

Things that I plan to add in the future:
- cify/uncify functions to turn string in memory to C and back to interact with C
- Universal IR subset to compile .o files into,
  so it would be possible to write b static libraries (.a),
  instead of sharing code just as source
  This subset should be just 1 or 2 byte target with functions to operate on it
  And linker and optimizer for each platform would be able to link and optimize that
- own linker and assembler for all targets
- `__asm__` blocks should not have string literals
  since we don't need c preprocessor compatibility
  and it will allow us not to escape `**` for mos6502 for example
  so we could just split by newlines
- Targets:
    - wasm for real
      maybe I could reuse code from original wasm target
    - jvm bytecode
    - dalvik bytecode
    - pdp-11
      just to test that it could run origiginal b compiler code
      and that universal subset works correctly with word pointer arithmetics
- fix btests for hist with each platform
