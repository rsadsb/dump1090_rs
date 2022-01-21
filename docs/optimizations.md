# `utils::to_mag`
## `RUSTFLAGS="-C target-cpu=native" cargo build --release`
### rizin-ghidra
```text
[0x0000c8f0]> pdg @ $(is~mag[1])

void sym.dump1090_rs::utils::to_mag::hc0809f05591c941a(void *arg1, int64_t arg2, int64_t arg3)
{
    undefined auVar1 [16];
    undefined auVar2 [16];
    undefined auVar3 [16];
    undefined auVar4 [16];
    undefined auVar5 [16];
    int64_t iVar6;
    int64_t iVar7;
    undefined4 uVar8;
    undefined auVar9 [16];
    undefined in_YMM3 [32];
    undefined in_XMM6 [16];

    // dump1090_rs::utils::to_mag::hc0809f05591c941a
    auVar9 = SUB3216(in_YMM3, 0);
    (*_reloc.memset)(arg1, 0, 0x4029c);
    if (arg3 != 0) {
        iVar6 = 0;
        auVar2 = vmovss_avx(*(undefined4 *)0xa1b04);
        auVar3 = vmovss_avx(*(undefined4 *)0xa1b08);
        auVar4 = vmovss_avx(*(undefined4 *)0xa1b0c);
        auVar9 = vxorps_avx(auVar9, auVar9);
        iVar7 = 0;
        do {
            if (0x20145 < iVar7 + 0x146U) {
                dbg.panic_bounds_check(iVar7 + 0x146U, 0x20146, 0xbf618);
                do {
                    invalidInstructionException();
                } while( true );
            }
            auVar1 = vcvtsi2ss_avx(in_XMM6, (int32_t)*(int16_t *)(arg2 + 2 + iVar6));
            auVar5 = vmulss_avx(auVar1, auVar2);
            auVar1 = vcvtsi2ss_avx(in_XMM6, (int32_t)*(int16_t *)(arg2 + iVar6));
            auVar1 = vmulss_avx(auVar1, auVar2);
            auVar5 = vmulss_avx(auVar5, auVar5);
            auVar1 = vmulss_avx(auVar1, auVar1);
            auVar1 = vaddss_avx(auVar5, auVar1);
            auVar1 = vsqrtss_avx(auVar1, auVar1);
            auVar1 = vmulss_avx(auVar1, auVar3);
            auVar1 = vaddss_avx(auVar1, auVar4);
            auVar1 = vmaxss_avx(auVar9, auVar1);
            auVar1 = vminss_avx(auVar3, auVar1);
            uVar8 = vcvttss2si_avx(auVar1);
            *(int16_t *)((int64_t)arg1 + iVar7 * 2 + 0x29c) = (int16_t)uVar8;
    // WARNING: Load size is inaccurate
            iVar7 = *arg1 + 1;
            *(int64_t *)arg1 = iVar7;
            iVar6 = iVar6 + 4;
        } while (arg3 << 2 != iVar6);
    }
    return;
}
```

## `cargo build --release`
### rizin-ghidra
```text
[0x0000cd50]> pdg @ $(is~mag[1])

void sym.dump1090_rs::utils::to_mag::h21b8408e85bb7f8c(void *arg1, int64_t arg2, int64_t arg3)
{
    float fVar1;
    float fVar2;
    float fVar3;
    int64_t iVar4;
    int64_t iVar5;
    float fVar6;
    float fVar7;

    // dump1090_rs::utils::to_mag::h21b8408e85bb7f8c
    (*_reloc.memset)(arg1, 0, 0x4029c);
    fVar3 = *(float *)0x9e854;
    fVar2 = *(float *)0x9e850;
    fVar1 = *(float *)0x9e84c;
    if (arg3 != 0) {
        iVar4 = 0;
        iVar5 = 0;
        do {
            if (0x20145 < iVar5 + 0x146U) {
                dbg.panic_bounds_check(iVar5 + 0x146U, 0x20146, 0xbc658);
                do {
                    invalidInstructionException();
                } while( true );
            }
            fVar6 = (float)(int32_t)*(int16_t *)(arg2 + 2 + iVar4) * fVar1;
            fVar7 = (float)(int32_t)*(int16_t *)(arg2 + iVar4) * fVar1;
            fVar7 = SQRT(fVar7 * fVar7 + fVar6 * fVar6) * fVar2 + fVar3;
            fVar6 = 0.0;
            if (0.0 <= fVar7) {
                fVar6 = fVar7;
            }
            fVar7 = fVar2;
            if (fVar6 <= fVar2) {
                fVar7 = fVar6;
            }
            *(int16_t *)((int64_t)arg1 + iVar5 * 2 + 0x29c) = (int16_t)(int32_t)fVar7;
    // WARNING: Load size is inaccurate
            iVar5 = *arg1 + 1;
            *(int64_t *)arg1 = iVar5;
            iVar4 = iVar4 + 4;
        } while (arg3 << 2 != iVar4);
    }
    return;
}
```
