# dump1090_rs
[![Actions Status](https://github.com/wcampbell0x2a/dump1090_rs/workflows/CI/badge.svg)](https://github.com/wcampbell0x2a/dump1090_rs/actions)

Fork of https://github.com/johnwstanford/dump1090_rs, without parsing messages.
This project is meant to just forward bytes from the the demodulated iq stream from a rtlsdr to my own [adsb_deku](https://github.com/wcampbell0x2a/adsb_deku) library/apps.

## Usage

```
cargo r --release
```

## Testing
```
cargo t --release
```

## Benchmark

Reading from a 256KB iq sample to ADS-B bytes takes ~3.2 ms, but feel free to run benchmarks on your computer.
```
cargo bench
```

### Performance tricks

To enable maximum performance, instruct rustc not to use cross cpu features only.
This gives around a .1% speedup in my tests.
```
RUSTFLAGS="-C target-cpu=native" cargo r --release
```

#### lscpu
```
> lscpu
Architecture:            x86_64
  CPU op-mode(s):        32-bit, 64-bit
  Address sizes:         39 bits physical, 48 bits virtual
  Byte Order:            Little Endian
CPU(s):                  8
  On-line CPU(s) list:   0-7
Vendor ID:               GenuineIntel
  Model name:            Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz
    CPU family:          6
    Model:               158
    Thread(s) per core:  2
    Core(s) per socket:  4
    Socket(s):           1
    Stepping:            9
    CPU max MHz:         4500.0000
    CPU min MHz:         800.0000
    BogoMIPS:            8403.00
    Flags:               fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid
                         aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault invpcid_single pti tpr_shadow vnmi fle
                         xpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 hle avx2 smep bmi2 erms invpcid rtm mpx rdseed adx smap clflushopt intel_pt xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp
```

# Changes
See [CHANGELOG.md](https://github.com/wcampbell0x2a/dump1090_rs/blob/master/CHANGELOG.md)
