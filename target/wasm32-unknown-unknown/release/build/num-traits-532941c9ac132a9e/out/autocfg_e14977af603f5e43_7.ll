; ModuleID = 'autocfg_e14977af603f5e43_7.eaa7b8a2351ca940-cgu.0'
source_filename = "autocfg_e14977af603f5e43_7.eaa7b8a2351ca940-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

; core::f64::<impl f64>::to_ne_bytes
; Function Attrs: inlinehint nounwind
define internal void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17hd22299638204f4fbE"(ptr sret([8 x i8]) align 1 %_0, double %self) unnamed_addr #0 {
start:
  %_2 = bitcast double %self to i64
  store i64 %_2, ptr %_0, align 1
  ret void
}

; autocfg_e14977af603f5e43_7::probe
; Function Attrs: nounwind
define dso_local void @_ZN26autocfg_e14977af603f5e43_75probe17hefb3ec003c335bf2E() unnamed_addr #1 {
start:
  %_1 = alloca [8 x i8], align 1
; call core::f64::<impl f64>::to_ne_bytes
  call void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17hd22299638204f4fbE"(ptr sret([8 x i8]) align 1 %_1, double 3.140000e+00) #2
  ret void
}

attributes #0 = { inlinehint nounwind "target-cpu"="generic" }
attributes #1 = { nounwind "target-cpu"="generic" }
attributes #2 = { nounwind }

!llvm.ident = !{!0}

!0 = !{!"rustc version 1.85.1 (4eb161250 2025-03-15)"}
