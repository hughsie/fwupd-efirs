# fwupd-efirs

A EFI Application used by uefi-capsule plugin in fwupd to update hardware.

## History

The fwupd-efi project was a small self-contained binary that parses EFI variables, calls half a
dozen EFI methods and also writes debugging to another EFI variable.
It is signed by the Red Hat signing key and is chainloaded by shim, so any security vulnerability
would be critical. It's a plain C superficial rewrite of code originally written by Peter Jones
and uses the increasingly unreliable and esoteric gnu-efi toolchain to build.

It needs to be built and signed for x64, i32 and aa64 and there are various firmware bugs to work
around needing the sections aligned in specific ways.

A rewrite-in-rust decreases the risk of using low-level C for parsing in safety critical code.
There is not a significant existing community of people helping with fwupd-efi (unlike the main
fwupd project) and so rewriting doesn’t come with the risk of alienating existing contributors.

## TODO

 [X] Build a stub EFI "hello world" program that can be started in a VM with unstable rust
 [X] Use uefi-rs to get and set UEFI variable, and contribute the code to delete them
 [ ] Add the capsule RT methods to uefi-rs
 [X] Switch to stable rust
 [ ] Use the new RT methods to actually schedule an capsule
 [ ] Test on my Lenovo Laptop on real hardware
 [ ] Test on non x64 machines
