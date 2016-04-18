# park_os

A toy OS (bootloader TBH) based upon/ripped off this fantastic series of blog posts:
http://os.phil-opp.com/multiboot-kernel.html

## Building (assuming Ubuntu Server 14.04 LTS)

```
sudo apt-get install nasm xorriso git qemu gdb
curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh
git clone https://github.com/zanders3/park_os
cd park_os
multirust override nightly
make run
```
